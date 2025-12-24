use std::thread;

#[derive(Debug, Clone)]
pub struct CPUInfo {
    pub logical_cores: usize,
    pub recommendation: String,
}

pub struct CPUDetector;

impl CPUDetector {
    pub fn detect() -> CPUInfo {
        let logical_cores = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        let recommendation = if logical_cores >= 16 {
            "High Performance: Use heavy multi-threading (Job System + Async Compute)".to_string()
        } else if logical_cores >= 8 {
            "Balanced: Standard multi-threading (Job System)".to_string()
        } else {
            "Low Power: Limit background threads".to_string()
        };

        println!("Detected CPU: {} Logical Cores", logical_cores);
        println!("Recommendation: {}", recommendation);

        CPUInfo {
            logical_cores,
            recommendation,
        }
    }
}
