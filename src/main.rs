use boids_sim::engine;

fn main() {
    pollster::block_on(engine::run());
}
