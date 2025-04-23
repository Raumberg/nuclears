use crossterm::event::KeyEvent;
use crate::system::SystemInfo;
use crate::reactor::Reactor;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

#[derive(PartialEq, Eq)]
pub enum AppState {
    Running,
    Paused,
}

pub struct App {
    pub state: AppState,
    pub system_info: SystemInfo,
    pub reactor: Reactor,
    pub show_help: bool,
    pub reactor_status: String,
    stress_thread: Option<StressThread>,
    simulation_active: bool,
    simulation_value: f32,
    simulation_direction: f32,
}

struct StressThread {
    handle: thread::JoinHandle<()>,
    stop_flag: Arc<AtomicBool>,
}

impl App {
    pub fn new() -> Self {
        App {
            state: AppState::Running,
            system_info: SystemInfo::new(),
            reactor: Reactor::new(),
            show_help: false,
            reactor_status: "Normal Operation".to_string(),
            stress_thread: None,
            simulation_active: false,
            simulation_value: 40.0,
            simulation_direction: 1.0,
        }
    }

    pub fn update(&mut self) {
        // Only update system info if the app is running
        if self.state == AppState::Running {
            self.system_info.update();
            
            // If simulation is active, override CPU usage with a smooth oscillating value
            if self.simulation_active {
                // Update simulation value to create a smooth wave pattern
                self.simulation_value += self.simulation_direction * 0.5;
                
                // Reverse direction at boundaries for oscillation
                if self.simulation_value > 95.0 {
                    self.simulation_value = 95.0;
                    self.simulation_direction = -1.0;
                } else if self.simulation_value < 20.0 {
                    self.simulation_value = 20.0;
                    self.simulation_direction = 1.0;
                }
                
                // Ensure CPU usage is always within valid range
                self.system_info.cpu_usage = self.simulation_value.max(0.0).min(100.0);
            }
            
            self.reactor.update(&self.system_info);
            
            // Update reactor status based on system load
            self.update_reactor_status();
        }
    }

    fn update_reactor_status(&mut self) {
        // If the reactor is exploding, set critical status
        if self.reactor.is_exploding {
            self.reactor_status = "CRITICAL - MELTDOWN IMMINENT!".to_string();
            return;
        }

        let stability = self.reactor.stability;
        self.reactor_status = match stability {
            s if s > 90.0 => "Critical - Meltdown Imminent!".to_string(),
            s if s > 75.0 => "Danger - Severe Radiation Leakage".to_string(),
            s if s > 60.0 => "Warning - Reactor Unstable".to_string(),
            s if s > 40.0 => "Caution - Increased Radiation Levels".to_string(),
            s if s > 20.0 => "Normal - Routine Operation".to_string(),
            _ => "Idle - Minimal Load".to_string(),
        };
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Char('p') => {
                self.toggle_pause();
            }
            crossterm::event::KeyCode::Char('h') => {
                self.show_help = !self.show_help;
            }
            crossterm::event::KeyCode::Char('s') => {
                self.toggle_stress_test();
            }
            _ => {}
        }
    }

    fn toggle_pause(&mut self) {
        self.state = match self.state {
            AppState::Running => AppState::Paused,
            AppState::Paused => AppState::Running,
        };
    }
    
    fn toggle_stress_test(&mut self) {
        if let Some(stress_thread) = &self.stress_thread {
            // Stop the stress test
            stress_thread.stop_flag.store(true, Ordering::SeqCst);
            self.stress_thread = None;
            self.simulation_active = true; // Keep showing high usage in UI
        } else {
            // Start the stress test or toggle simulation
            if self.simulation_active {
                self.simulation_active = false;
            } else {
                self.start_stress_test();
            }
        }
    }
    
    fn start_stress_test(&mut self) {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let thread_stop_flag = stop_flag.clone();
        
        let handle = thread::spawn(move || {
            // Create some CPU load by doing meaningless calculations
            while !thread_stop_flag.load(Ordering::SeqCst) {
                // Do some meaningless work to generate CPU load
                for _ in 0..10_000_000 {
                    let _result = 1 + 1;
                }
                
                // Short pause to prevent 100% CPU usage
                thread::sleep(Duration::from_millis(1));
            }
        });
        
        self.stress_thread = Some(StressThread {
            handle,
            stop_flag,
        });
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Make sure to terminate the stress test thread when the app exits
        if let Some(stress_thread) = &self.stress_thread {
            stress_thread.stop_flag.store(true, Ordering::SeqCst);
            // We intentionally don't join the thread here to avoid blocking
            // on app exit if the thread is somehow stuck
        }
    }
} 