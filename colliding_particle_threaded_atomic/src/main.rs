use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};
use scoped_threadpool::Pool;
use std::sync::atomic::{AtomicUsize, Ordering};
const NUM_OF_PARTICLES: usize = 100;
const ENCLOSURE_SIZE: i32 = 10;
const NUM_OF_THREADS: usize = 4;
const NUM_OF_CHUNKS: usize = 5;

#[derive(Debug, Copy, Clone)]
struct Particle {
    x: f32,
    y: f32,
}

impl Particle {
    pub fn new(x_param: f32, y_param: f32) -> Particle {
        Particle {
            x: x_param,
            y: y_param,
        }
    }

    pub fn move_randomly(&mut self) {
        let dx = (rand::random::<f32>() * 2.0 - 1.0).clamp(-1.0, 1.0);
        let dy = (rand::random::<f32>() * 2.0 - 1.0).clamp(-1.0, 1.0);
        self.x = (self.x + dx).clamp(0.0, (ENCLOSURE_SIZE - 1) as f32);
        self.y = (self.y + dy).clamp(0.0, (ENCLOSURE_SIZE - 1) as f32);
    }

    pub fn collide(&self, other: &Particle) -> bool {
        let distance = ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt();
        distance < 1.0
    }
}

struct ParticleSystem {
    particles: Arc<Mutex<Vec<Particle>>>,
    total_collisions: Arc<AtomicUsize>,
}

impl ParticleSystem {
    pub fn new() -> ParticleSystem {
        ParticleSystem {
            particles: Arc::new(Mutex::new(Vec::new())),
            total_collisions: Arc::new(AtomicUsize::new(0)),

        }
    }

    pub fn add_particle(&mut self, particle: Particle) {
        let mut particles = self.particles.lock().unwrap();
        if particles.len() < NUM_OF_PARTICLES {
            particles.push(particle);
        } else {
            println!("Particle Limit Reached!");
        }
    }

    pub fn simulate_particles(&self) -> (usize, usize) {
        let mut move_pool = Pool::new(NUM_OF_THREADS as u32);
        let mut collision_pool = Pool::new(NUM_OF_THREADS as u32); // Limiting to one thread initially
        let start_time = Instant::now();
        let duration_time = Duration::from_secs(10);
        let mut iterations = 0;

        while Instant::now() - start_time < duration_time {
            let particles = Arc::clone(&self.particles);
            let total_collisions = Arc::clone(&self.total_collisions);

            move_pool.scoped(|scope| {
                for i in 0..NUM_OF_THREADS {
                    let particles = Arc::clone(&particles);
                    scope.execute(move || {
                        let mut particles = particles.lock().unwrap();
                        for slice in particles.chunks_mut(NUM_OF_CHUNKS) {
                            thread_main_move(slice, i);
                        }
                    });
                }
            });

            collision_pool.scoped(|scope| {
                for i in 0..NUM_OF_THREADS {
                    let particles = Arc::clone(&particles);
                    let total_collisions = Arc::clone(&total_collisions);
                    scope.execute(move || {
                        let particles = particles.lock().unwrap();
                        
                        for slice in particles.chunks(NUM_OF_CHUNKS) {
                            thread_main_collisions(slice, i, &total_collisions);
                        }
                    });
                }
            });

            iterations += 1;
        }

        let final_collisions = self.total_collisions.load(Ordering::SeqCst);
        (iterations, final_collisions)
    }
}

fn thread_main_move(list: &mut [Particle], thread_id: usize) {
    for particle in list {
        particle.move_randomly();
     //   println!("Thread{}: New particle position at ({}, {})", thread_id, particle.x, particle.y);
    }
}

fn thread_main_collisions(list: &[Particle], thread_id: usize, total_collisions: &Arc<AtomicUsize>) {
    let mut collision_count = 0;
    for i in 0..list.len() {
        for j in (i + 1)..list.len() {
            if list[i].collide(&list[j]) {
                collision_count += 1;
                //println!("Collisions thread collisions: {}", collision_count);
            }
        }
    }
   // println!("Thread{}: Number of collisions: {}", thread_id, collision_count);
   total_collisions.fetch_add(collision_count, Ordering::SeqCst);
}

fn main() {
    let mut particle_system = ParticleSystem::new();
    for _ in 0..20 {
        let particle: Particle = Particle::new(5.0, 7.0);
        particle_system.add_particle(particle);
    }

    let (iterations, collisions) = particle_system.simulate_particles();
    println!("Number of iterations in 10 seconds: {}", iterations);
    println!("Total collisions detected during simulation: {}", collisions);
}