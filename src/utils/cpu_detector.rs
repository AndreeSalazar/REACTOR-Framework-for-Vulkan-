use std::thread;

#[derive(Debug, Clone)]
pub struct CPUInfo {
    pub logical_cores: usize,
    pub recommendation: String,
    pub thread_pool_size: usize,
}

pub struct CPUDetector;

impl CPUDetector {
    pub fn detect() -> CPUInfo {
        let logical_cores = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        let (recommendation, thread_pool_size) = if logical_cores >= 16 {
            (
                "High Performance: Use heavy multi-threading (Job System + Async Compute)".to_string(),
                logical_cores - 2, // Leave 2 cores for OS/main thread
            )
        } else if logical_cores >= 8 {
            (
                "Balanced: Standard multi-threading (Job System)".to_string(),
                logical_cores - 1,
            )
        } else if logical_cores >= 4 {
            (
                "Medium: Light multi-threading".to_string(),
                logical_cores / 2,
            )
        } else {
            (
                "Low Power: Limit background threads".to_string(),
                1,
            )
        };

        println!("Detected CPU: {} Logical Cores", logical_cores);
        println!("Recommendation: {}", recommendation);

        CPUInfo {
            logical_cores,
            recommendation,
            thread_pool_size,
        }
    }

    pub fn core_count() -> usize {
        thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
}
