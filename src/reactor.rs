use rand::Rng;
use std::collections::VecDeque;

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
        let mut rng = rand::thread_rng();
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        
        // Higher intensity increases particle speed and lifetime
        // Scale speed down for higher frame rate
        let base_speed = rng.gen_range(0.03..0.15) * FRAME_RATE_FACTOR;
        let speed = base_speed * (1.0 + intensity * 0.5);
        
        // Calculate lifetime with overflow protection
        // Ensure we stay within u8 bounds (0-255)
        let base_lifetime: u8 = rng.gen_range(60..180);
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
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        
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
    pub radiation_level: f32,
    pub core_temperature: f32,
    pub pressure: f32,
    pub rod_position: f32,  // 0.0 = fully inserted (low power), 1.0 = fully withdrawn (high power)
    pub instability: f32,
    pub history: VecDeque<f32>,
    pub particles: Vec<Particle>,
    pub rods_count: u8,
    pub coolant_level: f32,
    update_counter: u32,
    collisions_this_frame: usize,
    pub total_collisions: usize,
    pub is_exploding: bool,
    pub explosion_frame: u8,
}

impl Reactor {
    pub fn new() -> Self {
        Reactor {
            radiation_level: 10.0,
            core_temperature: 220.0,
            pressure: 101.3,
            rod_position: 0.3,
            instability: 0.0,
            history: VecDeque::with_capacity(HISTORY_SIZE),
            particles: Vec::with_capacity(MAX_PARTICLES),
            rods_count: 20,
            coolant_level: 95.0,
            update_counter: 0,
            collisions_this_frame: 0,
            total_collisions: 0,
            is_exploding: false,
            explosion_frame: 0,
        }
    }
    
    pub fn update(&mut self, cpu_load: f32) {
        self.update_counter = (self.update_counter + 1) % 1_000_000;
        self.collisions_this_frame = 0;
        
        // If reactor is exploding, advance explosion animation and skip normal updates
        if self.is_exploding {
            if self.update_counter % 5 == 0 { // Slow down animation
                if self.explosion_frame < 10 {
                    self.explosion_frame += 1;
                }
            }
            return;
        }
        
        // Update reactor parameters based on CPU load
        let target_rod_position = cpu_load / 100.0;
        
        // Simulate control rod movement (they move slowly)
        if (self.rod_position - target_rod_position).abs() > 0.01 {
            // Scale movement for higher frame rate
            self.rod_position += (target_rod_position - self.rod_position) * 0.03;
        }
        
        // Update reactor core temperature
        let target_temp = 220.0 + (700.0 * self.rod_position);
        // Scale temperature change for higher frame rate
        self.core_temperature += (target_temp - self.core_temperature) * 0.03;
        
        // Update radiation level
        self.radiation_level = clamp(10.0 + (90.0 * self.rod_position * self.rod_position), 10.0, 100.0);
        
        // Pressure increases with temperature
        self.pressure = clamp(101.3 + (self.core_temperature - 220.0) * 0.1, 100.0, 300.0);
        
        // Coolant is more depleted at higher temperatures
        self.coolant_level = clamp(100.0 - (self.core_temperature - 220.0) * 0.05, 0.0, 100.0);
        
        // Calculate instability (random fluctuations that increase with load)
        let mut rng = rand::thread_rng();
        let random_factor = rng.gen_range(-5.0..5.0);
        self.instability = (self.rod_position * 30.0) + random_factor;
        
        // Update history for graphs - update at original rate, not every frame
        if self.update_counter % 3 == 0 {
            if self.history.len() >= HISTORY_SIZE {
                self.history.pop_front();
            }
            self.history.push_back(self.core_temperature);
        }
        
        // Update existing particles
        self.particles.retain(|p| p.is_alive());
        for particle in &mut self.particles {
            particle.update();
        }
        
        // Check for collisions - we use indices to avoid borrow checker issues
        let mut new_particles = Vec::new();
        
        // We'll only check collision every few frames for performance
        if self.update_counter % 2 == 0 && self.particles.len() > 5 {
            for i in 0..self.particles.len() {
                for j in (i+1)..self.particles.len() {
                    if self.particles[i].collides_with(&self.particles[j]) {
                        // Collision detected! 
                        self.collisions_this_frame += 1;
                        
                        // Safety check for total_collisions to prevent overflow
                        if self.total_collisions < usize::MAX - 1 {
                            self.total_collisions += 1;
                        }
                        
                        // Each collision increases core temperature slightly
                        self.core_temperature = (self.core_temperature + 0.5).min(1000.0);
                        
                        // Ensure radiation level stays within bounds
                        self.radiation_level = clamp(self.radiation_level, 10.0, 100.0);
                        
                        // Ensure coolant level stays within bounds
                        self.coolant_level = clamp(self.coolant_level, 0.0, 100.0);
                        
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
                        let cpu_factor = clamp(cpu_load / 100.0, 0.0, 1.0);
                        let collision_chance = 0.3 + (cpu_factor * 0.4); // 30-70% chance
                        
                        if rng.r#gen::<f32>() < collision_chance && 
                            self.particles.len() + new_particles.len() < MAX_PARTICLES {
                            // Spawn 1-3 new particles from the collision
                            let spawn_count = ((cpu_factor * 3.0) as usize).min(3).max(1);
                            
                            for _ in 0..spawn_count {
                                if let (Some(p1), Some(p2)) = (self.particles.get(i), self.particles.get(j)) {
                                    new_particles.push(Particle::spawn_from_collision(p1, p2, &mut rng));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Add newly created particles from collisions
        self.particles.extend(new_particles);
        
        // Generate new particles based on radiation level and CPU load
        let cpu_factor = cpu_load / 100.0;
        let intensity = self.rod_position * cpu_factor;
        
        // Increase particle chance based on both radiation level and CPU load
        // Adjust particle generation rate for higher frame rate
        let particle_chance = ((self.radiation_level / 100.0) * 0.27) * (1.0 + cpu_factor);
        
        // Dynamic max particles based on CPU load
        let dynamic_max = (MAX_PARTICLES as f32 * (0.3 + 0.7 * cpu_factor)) as usize;
        
        // Generate multiple particles per frame at high load
        let particles_per_update = (1 + (cpu_factor * 3.0) as usize).min(4);
        
        for _ in 0..particles_per_update {
            if rng.r#gen::<f32>() < particle_chance && self.particles.len() < dynamic_max {
                // Generate particles near the core with more variation at higher loads
                let core_x = 0.5;
                let core_y = 0.5;
                let spread = 0.1 + (cpu_factor * 0.1);
                let offset_x = rng.gen_range(-spread..spread);
                let offset_y = rng.gen_range(-spread..spread);
                
                self.particles.push(Particle::new(
                    core_x + offset_x, 
                    core_y + offset_y,
                    intensity
                ));
            }
        }
        
        // Check if we've reached critical mass for explosion
        if self.total_collisions > 100 && self.stability() > 80.0 {
            self.is_exploding = true;
            self.explosion_frame = 0;
        }
    }
    
    pub fn stability(&self) -> f32 {
        // Calculate a stability score (0-100) where higher means more unstable
        let temp_factor = (self.core_temperature - 220.0) / 700.0;
        let rad_factor = self.radiation_level / 100.0;
        let coolant_factor = (100.0 - self.coolant_level) / 100.0;
        
        (temp_factor * 40.0 + rad_factor * 40.0 + self.instability + coolant_factor * 20.0).max(0.0).min(100.0)
    }
    
    pub fn collisions(&self) -> usize {
        self.collisions_this_frame
    }
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
} 