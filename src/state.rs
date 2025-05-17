use log::debug;
use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};
use crate::{boid::{generate_boids, triangle_buffer_layout, Boid, TRIANGLE_VERTICES}, vertex::{Vertex, VERTICES}};

pub struct State<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: &'a Window,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub num_instances: u32,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        // Its Purpose is to create Adapters and Surfaces 
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch="wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch="wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        // the part of window that we draw to.
        let surface = instance.create_surface(window).unwrap();

        // handle for our graphics card.
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        // create device and queue
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web, we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            },
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        // load in the shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("boid_vs_main"), // 1.
                buffers: &[
                    // boid instance buffer layout
                    Boid::desc(),
                    //shared triangle buffer layout
                    triangle_buffer_layout(),
                ], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: Some("boid_fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None, // 6.
        });

        const POPULATION : u32 = 100;
        let boids = generate_boids(POPULATION);
        debug!("{:?}", boids);
        // shared vertex buffer across all boids.
        // Since each boid is essentially a triangle, we will redraw this one triangle instance N times,
        // each with different parameters from the boids array
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("vertex buffer"),
                contents: bytemuck::cast_slice(&TRIANGLE_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        
        // This is the boid instance buffer, which contains the information of the boids (position & velocity)
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&boids),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let num_vertices = TRIANGLE_VERTICES.len() as u32;
        let num_instances = boids.len() as u32;
        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            num_vertices,
            num_instances,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            debug!("Resizing to {0}x{1}", new_size.width, new_size.height);

            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("RenderEncoder"),
        });
        // clearing the screen
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                    view: &view, 
                    resolve_target: None, 
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { 
                            r: 0.3, 
                            g: 0.0, 
                            b: 0.075, 
                            a: 1.0 
                        }),
                        store: wgpu::StoreOp::Store,

                    },
                })],
                depth_stencil_attachment:None,
                occlusion_query_set:None,
                timestamp_writes: None,
            });
            
            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));        // N boids
            render_pass.set_vertex_buffer(1, self.vertex_buffer.slice(..));

            render_pass.draw(0..3, 0..self.num_instances as u32); // 3 vertices, N instances
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
 