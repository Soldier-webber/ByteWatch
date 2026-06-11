/// System information monitoring module
/// 
/// This module provides system statistics collection using sysinfo.
/// It handles:
/// - CPU usage
/// - Memory usage
/// - Disk usage
/// - Network throughput
/// - Process information

use sysinfo::{System, SystemExt, ProcessExt, NetworkExt, DiskExt};

/// Information about a single process
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// CPU usage percentage (0-100)
    pub cpu_usage: f32,
    /// Memory usage in bytes
    pub memory_usage: u64,
}

/// System monitor for collecting real-time statistics
pub struct SystemMonitor {
    /// sysinfo System instance
    system: System,
    /// Previous network stats for throughput calculation
    prev_received: u64,
    prev_transmitted: u64,
}

impl SystemMonitor {
    /// Create a new system monitor instance
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system,
            prev_received: 0,
            prev_transmitted: 0,
        }
    }

    /// Refresh system statistics
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    /// Get current CPU usage as a percentage (0-100)
    pub fn cpu_usage(&self) -> f32 {
        // Calculate average CPU usage across all processors
        let cpus = self.system.cpus();
        if cpus.is_empty() {
            return 0.0;
        }

        let total: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
        (total / cpus.len() as f32).min(100.0).max(0.0)
    }

    /// Get current memory usage as a percentage (0-100)
    pub fn memory_usage(&self) -> f32 {
        let total_memory = self.system.total_memory();
        if total_memory == 0 {
            return 0.0;
        }

        let used_memory = self.system.used_memory();
        ((used_memory as f32 / total_memory as f32) * 100.0)
            .min(100.0)
            .max(0.0)
    }

    /// Get memory info as (used, total) in bytes
    pub fn memory_info(&self) -> (u64, u64) {
        (
            self.system.used_memory(),
            self.system.total_memory(),
        )
    }

    /// Get disk usage info as (used, total) in bytes
    /// Returns the root filesystem or the first mounted filesystem
    pub fn disk_usage(&self) -> (u64, u64) {
        let disks = self.system.disks();
        
        // Try to find root filesystem first, otherwise use the first disk
        let disk = disks
            .iter()
            .find(|d| d.mount_point() == std::path::Path::new("/"))
            .or_else(|| disks.first())
            .map(|d| {
                let total = d.total_space();
                let available = d.available_space();
                let used = total.saturating_sub(available);
                (used, total)
            })
            .unwrap_or((0, 0));

        disk
    }

    /// Get network throughput as (received, transmitted) in bytes per second
    pub fn network_throughput(&self) -> (u64, u64) {
        let networks = self.system.networks();
        
        let received: u64 = networks.iter().map(|(_, data)| data.received()).sum();
        let transmitted: u64 = networks.iter().map(|(_, data)| data.transmitted()).sum();

        // Calculate throughput (bytes per second)
        let recv_diff = received.saturating_sub(self.prev_received);
        let trans_diff = transmitted.saturating_sub(self.prev_transmitted);

        (recv_diff, trans_diff)
    }

    /// Update network stats for next throughput calculation
    fn update_network_stats(&mut self) {
        let networks = self.system.networks();
        
        self.prev_received = networks.iter().map(|(_, data)| data.received()).sum();
        self.prev_transmitted = networks.iter().map(|(_, data)| data.transmitted()).sum();
    }

    /// Get list of processes sorted by CPU usage
    pub fn get_processes(&self) -> Vec<ProcessInfo> {
        self.system
            .processes()
            .iter()
            .map(|(pid, process)| ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
            })
            .collect()
    }

    /// Get total number of processes
    pub fn process_count(&self) -> usize {
        self.system.processes().len()
    }

    /// Get system uptime in seconds
    pub fn uptime(&self) -> u64 {
        System::uptime()
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for formatting system information

/// Format bytes to human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Format seconds to human-readable uptime string
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(30), "0m");
        assert_eq!(format_uptime(3600), "1h 0m");
        assert_eq!(format_uptime(90061), "1d 1h 1m");
    }
}
