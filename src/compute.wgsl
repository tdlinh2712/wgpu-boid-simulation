// Compute shader

// Boid struct that matches buffer in rust

struct Boid {
    pos: vec2<f32>,
    vel: vec2<f32>,
}

// constan values -> TODO: move this to parameters
const DISTANCE : f32 = 0.4;
const DELTA_T : f32 = 0.02; // smaller step, smoother motion
const SEPARATION_DISTANCE : f32 = 0.04; // smaller separation distance 
const COHESION_WEIGHT : f32 = 0.2; // gentle pull towar center 
const ALIGNMENT_WEIGHT : f32 = 0.3; // a bit more to match velocity
const SEPARATION_WEIGHT : f32 = 0.8; // stronger force to avoid overlap
const MAX_SPEED : f32 = 0.2; // keep boids from moving too fast
// Storage buffer - input. out

@group(0) @binding(0)
var<storage, read> boid_in: array<Boid>;
@group(0) @binding(1)
var<storage, read_write> boid_out: array<Boid>;

// compute entry point

// workgroup_size tells the dimension of the workgroup's local grid of invocation
// each workgroup has 64 threads
@compute @workgroup_size(64) 
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let total = arrayLength(&boid_in);
    if (i >= total) {
        return;
    }
    let current_boid = boid_in[i];
    var count : f32 = 0.0;
    var avg_alignment = vec2<f32>(0.0, 0.0);
    var avg_cohesion = vec2<f32>(0.0, 0.0);
    var avg_separation = vec2<f32>(0.0, 0.0);

    for (var j: u32 = 0u; j < total; j++) {
        if (i == j) {
            // ignore its own boid
            continue; 
        }

        let other = boid_in[j];
        let dist = distance(other.pos, current_boid.pos);
        if (dist > DISTANCE) {
            continue;
        }
        if (dist <= SEPARATION_DISTANCE) {
        // separation: remove avg position of surrounding boids
        avg_separation -= (other.pos - current_boid.pos); 
        }
        // Alignment : add avg velocity of the surrounding boids
        avg_alignment += other.vel;

        // Cohesion: add avg position of surrounding boids
        avg_cohesion += other.pos;

        count += 1.0;
    }
    // divide forces by count and limit speed
    if (count > 0.0) {
        // avg_separation = normalize(avg_separation / count);
        avg_alignment = normalize(avg_alignment / count);
        avg_cohesion = normalize(avg_cohesion / count) - current_boid.pos;
    }
    let acc = avg_cohesion * COHESION_WEIGHT + avg_alignment * ALIGNMENT_WEIGHT + avg_separation * SEPARATION_WEIGHT;
    var vel = current_boid.vel + acc;
    
    // if (length(vel) > max_speed) {
    //     vel = normalize(vel) * max_speed;
    // }
    vel = normalize(vel) * clamp(length(vel), 0.0, MAX_SPEED);
    var pos = current_boid.pos + (vel * DELTA_T);

    // limit by the screen
    if (pos.x > 1.0) {
        pos.x = -1.0;
    }
    if (pos.x <-1.0) {
        pos.x = 1.0;
    }
    if (pos.y > 1.0) {
        pos.y = -1.0;
    }
    if (pos.y <-1.0) {
        pos.y = 1.0;
    }

    boid_out[i] = Boid(pos, vel);
}


