use sysinfo::{System, RefreshKind};

pub struct SystemInfo {
    sys: System,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub cpu_temp: f32,
    pub uptime: u64,
    pub running_processes: usize,
}

impl SystemInfo {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::everything()
        );
        sys.refresh_all();
        
        SystemInfo {
            sys,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            cpu_temp: 0.0,
            uptime: 0,
            running_processes: 0,
        }
    }

    pub fn update(&mut self) {
        self.sys.refresh_all();
        
        // Calculate average CPU usage
        let mut cpu_usage_total = 0.0;
        let cpu_count = self.sys.cpus().len();
        
        for cpu in self.sys.cpus() {
            cpu_usage_total += cpu.cpu_usage();
        }
        
        self.cpu_usage = cpu_usage_total / cpu_count as f32;
        
        // Memory usage percentage
        let total_memory = self.sys.total_memory();
        let used_memory = self.sys.used_memory();
        
        self.memory_usage = (used_memory as f32 / total_memory as f32) * 100.0;
        
        // Get system uptime
        self.uptime = System::uptime();
        
        // Count running processes
        self.running_processes = self.sys.processes().len();
        
        // Simulate temperature (sysinfo doesn't provide temp on all platforms)
        // In a real app, you might use another crate or platform-specific code
        self.cpu_temp = 40.0 + (self.cpu_usage * 0.5);
    }
} 