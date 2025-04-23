use rand::Rng;
use rand::rngs::ThreadRng;
use std::collections::VecDeque;
use num_cpus;
use crate::system::SystemInfo;

const HISTORY_SIZE: usize = 30;
const MAX_PARTICLES: usize = 200;
const FRAME_RATE_FACTOR: f32 = 0.33;
const COLLISION_RADIUS: f32 = 0.02;

#[derive(Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32, 
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub lifetime: u8,
    pub radius: f32,
    pub energy: f32,
}

impl Particle {
    fn new(x: f32, y: f32, intensity: f32) -> Self {
        let mut rng = rand::rng();
        let angle = rng.r#gen_range(0.0..std::f32::consts::TAU);
        
        // Higher intensity increases particle speed and lifetime
        // Scale speed down for higher frame rate
        let base_speed = rng.r#gen_range(0.03..0.15) * FRAME_RATE_FACTOR;
        let speed = base_speed * (1.0 + intensity * 0.5);
        
        // Calculate lifetime with overflow protection
        // Ensure we stay within u8 bounds (0-255)
        let base_lifetime: u8 = rng.r#gen_range(60..180);
        let lifetime_boost: u8 = (intensity * 40.0).min(75.0) as u8;
        // Use saturating_add to prevent overflow
        let lifetime = base_lifetime.saturating_add(lifetime_boost);
        
        // Higher intensity = more energetic particles
        let energy = 0.5 + (intensity * 0.5);
        
        Particle {
            x,
            y,
            velocity_x: angle.cos() * speed,
            velocity_y: angle.sin() * speed,
            lifetime,
            radius: COLLISION_RADIUS,
            energy,
        }
    }
    
    fn update(&mut self) {
        self.x += self.velocity_x;
        self.y += self.velocity_y;
        
        // Bounce off walls
        if self.x < self.radius || self.x > 1.0 - self.radius {
            self.velocity_x = -self.velocity_x * 0.8; // Dampening factor
            
            // Ensure particle stays within bounds
            if self.x < self.radius {
                self.x = self.radius;
            } else {
                self.x = 1.0 - self.radius;
            }
        }
        
        if self.y < self.radius || self.y > 1.0 - self.radius {
            self.velocity_y = -self.velocity_y * 0.8; // Dampening factor
            
            // Ensure particle stays within bounds
            if self.y < self.radius {
                self.y = self.radius;
            } else {
                self.y = 1.0 - self.radius;
            }
        }
        
        if self.lifetime > 0 {
            self.lifetime -= 1;
        }
    }
    
    pub fn is_alive(&self) -> bool {
        self.lifetime > 0
    }
    
    pub fn collides_with(&self, other: &Particle) -> bool {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let distance_squared = dx * dx + dy * dy;
        distance_squared < (self.radius + other.radius).powi(2)
    }
    
    // Create a new particle from a collision
    pub fn spawn_from_collision(p1: &Particle, p2: &Particle, rng: &mut impl Rng) -> Self {
        // New particle spawns at the midpoint of collision
        let mid_x = (p1.x + p2.x) / 2.0;
        let mid_y = (p1.y + p2.y) / 2.0;
        
        // Random angle for velocity
        let angle = rng.r#gen_range(0.0..std::f32::consts::TAU);
        
        // Combined energy creates more energetic new particles
        let combined_energy = (p1.energy + p2.energy) * 0.6;
        let speed = 0.05 * FRAME_RATE_FACTOR * combined_energy;
        
        // New lifetime is average of parents
        let lifetime = ((p1.lifetime as u16 + p2.lifetime as u16) / 3) as u8;
        
        Particle {
            x: mid_x,
            y: mid_y,
            velocity_x: angle.cos() * speed,
            velocity_y: angle.sin() * speed,
            lifetime,
            radius: COLLISION_RADIUS,
            energy: combined_energy,
        }
    }
}

pub struct Reactor {
    pub particles: Vec<Particle>,
    pub radiation_level: f32,
    pub core_temperature: f32,
    pub coolant_level: f32,
    pub control_rods: f32,
    pub stability: f32,
    pub collision_count: u32,
    pub core_loadings: Vec<f32>, // Store per-core load for visualization
    pub is_exploding: bool,
    pub rod_position: f32,
    pub explosion_frame: u32,
    pub total_collisions: u32,
    pub history: Vec<f32>,
    rng: ThreadRng,
}

impl Reactor {
    pub fn new() -> Self {
        let num_cores = num_cpus::get();
        
        Reactor {
            particles: Vec::with_capacity(MAX_PARTICLES),
            radiation_level: 50.0,
            core_temperature: 50.0,
            coolant_level: 80.0,
            control_rods: 50.0,
            stability: 100.0,
            collision_count: 0,
            core_loadings: vec![0.0; num_cores], // Initialize with zeros for each core
            is_exploding: false,
            rod_position: 0.5,
            explosion_frame: 0,
            total_collisions: 0,
            history: vec![0.0; HISTORY_SIZE],
            rng: rand::rng(),
        }
    }
    
    pub fn update(&mut self, system_info: &SystemInfo) {
        // Update per-core load information
        for (i, load) in self.core_loadings.iter_mut().enumerate() {
            if i < system_info.core_usage.len() {
                *load = system_info.get_core_usage(i);
            }
        }
        
        // Update reactor parameters based on system info
        self.radiation_level = (system_info.cpu_usage / 100.0) * 100.0;
        
        // Adjust core temperature based on CPU usage and cooling
        let target_temp = 50.0 + (system_info.cpu_usage / 100.0) * 50.0;
        self.core_temperature = self.core_temperature * 0.9 + target_temp * 0.1;
        
        // Adjust coolant based on temperature
        self.coolant_level = self.coolant_level * 0.99 + (100.0 - self.core_temperature * 0.5) * 0.01;
        self.coolant_level = self.coolant_level.clamp(10.0, 100.0);
        
        // Calculate stability based on various factors
        self.stability = 100.0 
            - (self.core_temperature - 50.0).abs() * 0.5 
            - (self.radiation_level - 50.0).abs() * 0.3
            - (self.coolant_level - 80.0).abs() * 0.2;
        self.stability = self.stability.clamp(0.0, 100.0);
        
        // Adjust control rods based on stability
        if self.stability < 70.0 {
            self.control_rods += (90.0 - self.control_rods) * 0.1;
        } else if self.core_temperature > 70.0 {
            self.control_rods += (90.0 - self.control_rods) * 0.05;
        } else {
            self.control_rods -= (self.control_rods - 20.0) * 0.01;
        }
        self.control_rods = self.control_rods.clamp(0.0, 100.0);
        
        // Update particles
        self.update_particles();
        
        // Spawn new particles based on radiation level
        let spawn_chance = self.radiation_level / 100.0;
        if self.rng.r#gen::<f32>() < spawn_chance && self.particles.len() < MAX_PARTICLES {
            self.spawn_particle();
        }
    }
    
    // Get the load for a specific core
    pub fn get_core_load(&self, core_index: usize) -> f32 {
        if core_index < self.core_loadings.len() {
            self.core_loadings[core_index]
        } else {
            0.0
        }
    }
    
    pub fn update_particles(&mut self) {
        // Update existing particles
        self.particles.retain(|p| p.is_alive());
        for particle in &mut self.particles {
            particle.update();
        }
        
        // Check for collisions - we use indices to avoid borrow checker issues
        let mut new_particles = Vec::new();
        
        // We'll only check collision every few frames for performance
        if self.rng.r#gen::<f32>() < 0.01 && self.particles.len() > 5 {
            for i in 0..self.particles.len() {
                for j in (i+1)..self.particles.len() {
                    if self.particles[i].collides_with(&self.particles[j]) {
                        // Collision detected! 
                        self.collision_count += 1;
                        
                        // Particles bounce off each other
                        if let Some(p1) = self.particles.get_mut(i) {
                            p1.velocity_x = -p1.velocity_x;
                            p1.velocity_y = -p1.velocity_y;
                        }
                        
                        if let Some(p2) = self.particles.get_mut(j) {
                            p2.velocity_x = -p2.velocity_x;
                            p2.velocity_y = -p2.velocity_y;
                        }
                        
                        // Only create new particles if we're not at capacity and chance permits
                        let collision_chance = 0.3; // 30% chance
                        
                        if self.rng.r#gen::<f32>() < collision_chance && 
                            self.particles.len() + new_particles.len() < MAX_PARTICLES {
                            // Spawn 1-3 new particles from the collision
                            let spawn_count = ((self.rng.r#gen::<f32>() * 3.0) as usize).min(3).max(1);
                            
                            for _ in 0..spawn_count {
                                if let (Some(p1), Some(p2)) = (self.particles.get(i), self.particles.get(j)) {
                                    new_particles.push(Particle::spawn_from_collision(p1, p2, &mut self.rng));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Add newly created particles from collisions
        self.particles.extend(new_particles);
    }
    
    pub fn spawn_particle(&mut self) {
        // Generate a new particle near the core with more variation
        let core_x = 0.5;
        let core_y = 0.5;
        let spread = 0.1;
        let offset_x = self.rng.r#gen_range(-spread..spread);
        let offset_y = self.rng.r#gen_range(-spread..spread);
        
        self.particles.push(Particle::new(
            core_x + offset_x, 
            core_y + offset_y,
            self.radiation_level / 100.0
        ));
    }
    
    // Add a collisions method that returns the recent collision count
    pub fn collisions(&self) -> u32 {
        self.collision_count
    }
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
} 