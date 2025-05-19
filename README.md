# Boids Simulation

A real-time implementation of Craig Reynolds' Boids algorithm using wgpu. This simulation demonstrates emergent flocking behavior through simple rules of separation, alignment, and cohesion.

## Description

This project simulates the flocking behavior of birds (or "boids") using three fundamental rules:
- **Separation**: Avoid crowding neighbors
- **Alignment**: Steer towards average heading of neighbors
- **Cohesion**: Steer towards average position of neighbors

The simulation runs on the GPU using WebGPU for efficient parallel processing of boid behaviors.

## Tech Stack

- **Rust**: Core programming language
- **Winit**: Cross-platform window creation and event handling
- **WASM**: WebAssembly support for running in browsers
- **Wgpu**: Rust implementation of WebGPU

## Features

- Real-time simulation of up to 50,000 boids
- GPU-accelerated computation
- Cross-platform support (Desktop and Web)
- Smooth flocking behavior with configurable parameters
- FPS counter for performance monitoring

## Prerequisites

- Rust (latest stable version)
- For WebAssembly support:
  - wasm-pack
  - wasm-bindgen
  - web-sys

## Running the Project

### Desktop

1. Clone the repository:
```bash
git clone https://github.com/yourusername/boids_sim.git
cd boids_sim
```

2. Build and run:
```bash
cargo run
```

### Web (WASM)

1. Install wasm-pack if you haven't already:
```bash
cargo install wasm-pack
```

2. Build for web:
```bash
wasm-pack build --target web
```

3. Serve the files using a local server (e.g., using Python):
```bash
python -m http.server
```

4. Open your browser and navigate to `http://localhost:8000`

## Controls

- **ESC**: Exit the application
- The simulation automatically wraps around screen edges

## Performance

The simulation is optimized for GPU computation, with the following features:
- Compute shader for boid behavior calculations
- Instance rendering for efficient boid visualization
- Configurable parameters for flocking behavior

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 