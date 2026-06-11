/// Application state management
/// 
/// This module manages the overall application state, including:
/// - System statistics
/// - Process list and sorting
/// - Configuration

use crate::system::{SystemMonitor, ProcessInfo};

/// Sorting order for processes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    /// Sort by CPU usage (default)
    Cpu,
    /// Sort by memory usage
    Memory,
    /// Sort by process name
    Name,
    /// Sort by process ID
    Pid,
}

/// Main application state
pub struct App {
    /// System monitor instance
    system_monitor: SystemMonitor,
    /// List of processes sorted by the current sort order
    processes: Vec<ProcessInfo>,
    /// Current sort order
    sort_by: SortBy,
    /// Whether to refresh immediately on next update
    refresh_immediately: bool,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        let mut monitor = SystemMonitor::new();
        let processes = monitor.get_processes();

        Self {
            system_monitor: monitor,
            processes,
            sort_by: SortBy::Cpu,
            refresh_immediately: false,
        }
    }

    /// Update application state (called on each tick)
    pub fn update(&mut self) {
        // Refresh system statistics
        self.system_monitor.refresh();
        
        // Get updated process list
        self.processes = self.system_monitor.get_processes();
        
        // Sort processes according to current sort order
        self.sort_processes();
    }

    /// Trigger immediate refresh on next update
    pub fn refresh_immediately(&mut self) {
        self.refresh_immediately = true;
    }

    /// Sort the process list according to the current sort order
    fn sort_processes(&mut self) {
        match self.sort_by {
            SortBy::Cpu => {
                self.processes.sort_by(|a, b| {
                    b.cpu_usage.partial_cmp(&a.cpu_usage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortBy::Memory => {
                self.processes.sort_by(|a, b| {
                    b.memory_usage.cmp(&a.memory_usage)
                });
            }
            SortBy::Name => {
                self.processes.sort_by(|a, b| {
                    a.name.cmp(&b.name)
                });
            }
            SortBy::Pid => {
                self.processes.sort_by(|a, b| {
                    a.pid.cmp(&b.pid)
                });
            }
        }
    }

    // Getters for UI rendering

    /// Get current CPU usage percentage
    pub fn cpu_usage(&self) -> f32 {
        self.system_monitor.cpu_usage()
    }

    /// Get current memory usage percentage
    pub fn memory_usage(&self) -> f32 {
        self.system_monitor.memory_usage()
    }

    /// Get memory info (used, total) in bytes
    pub fn memory_info(&self) -> (u64, u64) {
        self.system_monitor.memory_info()
    }

    /// Get disk usage info (used, total) in bytes
    pub fn disk_usage(&self) -> (u64, u64) {
        self.system_monitor.disk_usage()
    }

    /// Get network throughput (received, transmitted) in bytes
    pub fn network_throughput(&self) -> (u64, u64) {
        self.system_monitor.network_throughput()
    }

    /// Get the process list
    pub fn processes(&self) -> &[ProcessInfo] {
        &self.processes
    }

    /// Get the current sort order
    pub fn sort_by(&self) -> SortBy {
        self.sort_by
    }

    /// Get total number of processes
    pub fn process_count(&self) -> usize {
        self.system_monitor.process_count()
    }

    /// Get system uptime in seconds
    pub fn uptime(&self) -> u64 {
        self.system_monitor.uptime()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
