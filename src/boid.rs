use rand::Rng;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Boid {
    pos: [f32; 2],
    vel: [f32; 2],
}

pub fn generate_boids(population: u32) -> Vec<Boid> { 
    let mut rng = rand::thread_rng();

    (0..population)
    .map(|_| {
        // initializing velocities to be pointing at random directions
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(0.005..0.015);
        // constraint intial positions to be in a radius

        let r = rng.gen_range(0.1..0.7); // radial distance
        let theta = rng.gen_range(0.0..std::f32::consts::TAU);
        
        Boid { 
            pos:[r * theta.cos(), r * theta.sin()],
            vel: [angle.cos() * speed, angle.sin() * speed],      
        }}
    ).collect()
}

impl Boid {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Boid>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub const TRIANGLE_VERTICES: [[f32; 2]; 3] = [
    [0.0, 0.0075],
    [-0.00375, -0.005],  
    [0.00375,  -0.005],
];

pub fn triangle_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x2,
        }],
    }
}