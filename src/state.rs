use std::time::{Duration, Instant};

use log::debug;
use wgpu::{util::DeviceExt, ComputePipelineDescriptor, Instance, PipelineLayoutDescriptor};
use winit::{event::WindowEvent, window::Window};
use crate::{boid::{generate_boids, triangle_buffer_layout, Boid, TRIANGLE_VERTICES}, vertex::{Vertex, VERTICES}};

struct Fps {
    frame_num: usize,
    last_frame_num: usize,
    last_fps_time: Instant,
}

pub struct State<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: &'a Window,
    pub render_pipeline: wgpu::RenderPipeline,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub num_vertices: u32,
    pub num_instances: u32,
    pub vertex_buffer: wgpu::Buffer,
    pub instance_buffers: Vec<wgpu::Buffer>,
    pub compute_bind_groups: Vec<wgpu::BindGroup>,
    pub fps: Fps,
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
        
        const POPULATION : u32 = 40000;
        let boids = generate_boids(POPULATION);
        debug!("{:?}", boids);
        // This is the boid instance buffer, which contains the information of the boids (position & velocity)
        let num_vertices = TRIANGLE_VERTICES.len() as u32;
        let num_instances = boids.len() as u32;

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

        // load compute shader
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compute.wgsl").into()),
        });

        let bind_group_layout  =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new((num_instances * 16) as _), //16 bytes is the size of Boid struct
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new((num_instances * 16) as _),
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("compute"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
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

        let mut instance_buffers : Vec<wgpu::Buffer> = Vec::<wgpu::Buffer>::new();

        for i in 0..2 {
            instance_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&boids),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
                })
            )
        };
        let mut compute_bind_groups : Vec<wgpu::BindGroup> = Vec::<wgpu::BindGroup>::new();
        for i in 0..2 {
            compute_bind_groups.push(
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("compute bind group {}", i)),
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: instance_buffers[i].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: instance_buffers[(i + 1) % 2].as_entire_binding(), // bind to opposite buffer
                        },
                    ]
                })
            )
        }
        let fps = Fps {
            frame_num: 0,
            last_frame_num: 0,
            last_fps_time: Instant::now(),
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            compute_pipeline,
            num_vertices,
            num_instances,
            vertex_buffer,
            instance_buffers,
            compute_bind_groups,
            fps,
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
        // compute pass
        {  
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"), 
                timestamp_writes: None 
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0,&self.compute_bind_groups[self.fps.frame_num % 2], &[]);
            compute_pass.dispatch_workgroups((self.num_instances + 63) / 64, 1, 1);
        }
        // render pass
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
            
            render_pass.set_vertex_buffer(0, self.instance_buffers[(self.fps.frame_num + 1) % 2].slice(..));        // N boids
            render_pass.set_vertex_buffer(1, self.vertex_buffer.slice(..));

            render_pass.draw(0..3, 0..self.num_instances as u32); // 3 vertices, N instances
        }
        self.fps.frame_num+=1;

        let now = Instant::now();
        let elapsed = now.duration_since(self.fps.last_fps_time);

        if elapsed >= Duration::from_secs(1) {
            let frame_count = self.fps.frame_num - self.fps.last_frame_num;
            let fps = frame_count as f32 / elapsed.as_secs_f32();
            println!("FPS: {:.2}", fps);

            self.fps.last_fps_time = now;
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
 