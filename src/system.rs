use std::time::{Duration, Instant};
use sysinfo::{System, CpuRefreshKind};

pub struct SystemInfo {
    sys: System,
    pub uptime: Duration,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub cpu_count: usize,
    pub cpu_temp: f32,
    pub core_usage: Vec<f32>,
    start_time: Instant,
}

impl SystemInfo {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let cpu_count = sys.cpus().len();
        let mut core_usage = vec![0.0; cpu_count];
        
        for (i, cpu) in sys.cpus().iter().enumerate() {
            core_usage[i] = cpu.cpu_usage();
        }
        
        SystemInfo {
            sys,
            uptime: Duration::from_secs(0),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_total: 0,
            memory_used: 0,
            cpu_count,
            cpu_temp: 20.0,
            core_usage,
            start_time: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        self.sys.refresh_all();
        
        self.uptime = self.start_time.elapsed();
        
        let mut total_usage = 0.0;
        for (i, cpu) in self.sys.cpus().iter().enumerate() {
            let usage = cpu.cpu_usage();
            self.core_usage[i] = usage;
            total_usage += usage;
        }
        
        self.cpu_usage = total_usage / self.cpu_count as f32;
        
        self.memory_total = self.sys.total_memory();
        self.memory_used = self.sys.used_memory();
        self.memory_usage = (self.memory_used as f32 / self.memory_total as f32) * 100.0;
        
        self.cpu_temp = 40.0 + (self.cpu_usage / 100.0) * 60.0;
    }

    pub fn get_core_usage(&self, core_index: usize) -> f32 {
        if core_index < self.core_usage.len() {
            self.core_usage[core_index]
        } else {
            self.cpu_usage
        }
    }
} 