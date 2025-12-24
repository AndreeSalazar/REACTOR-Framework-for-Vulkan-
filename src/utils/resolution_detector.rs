use winit::event_loop::ActiveEventLoop;

pub struct ResolutionDetector;

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub name: String,
    pub physical_width: u32,
    pub physical_height: u32,
    pub logical_width: f64,
    pub logical_height: f64,
    pub scale_factor: f64,
    pub refresh_rate: Option<u32>,
}

impl ResolutionDetector {
    pub fn get_smart_resolution(
        event_loop: &ActiveEventLoop,
        target_width: f32,
        target_height: f32,
    ) -> (f64, f64) {
        let monitor = event_loop.primary_monitor().or_else(|| event_loop.available_monitors().next());
        
        if let Some(monitor) = monitor {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            
            let mon_width_logical = size.width as f64 / scale;
            let mon_height_logical = size.height as f64 / scale;
            
            println!("Detected Monitor: {} ({:?})", monitor.name().unwrap_or_default(), size);
            println!("  Scale Factor: {}", scale);
            println!("  Logical Size: {}x{}", mon_width_logical, mon_height_logical);

            let w = if (target_width as f64) > mon_width_logical { mon_width_logical } else { target_width as f64 };
            let h = if (target_height as f64) > mon_height_logical { mon_height_logical } else { target_height as f64 };
            
            println!("  Selected Resolution: {}x{}", w, h);
            (w, h)
        } else {
            println!("No monitor detected, using default target.");
            (target_width as f64, target_height as f64)
        }
    }

    pub fn get_primary_monitor_info(event_loop: &ActiveEventLoop) -> Option<MonitorInfo> {
        let monitor = event_loop.primary_monitor().or_else(|| event_loop.available_monitors().next())?;
        
        let size = monitor.size();
        let scale = monitor.scale_factor();
        
        Some(MonitorInfo {
            name: monitor.name().unwrap_or_default(),
            physical_width: size.width,
            physical_height: size.height,
            logical_width: size.width as f64 / scale,
            logical_height: size.height as f64 / scale,
            scale_factor: scale,
            refresh_rate: monitor.refresh_rate_millihertz().map(|r| r / 1000),
        })
    }

    pub fn get_all_monitors(event_loop: &ActiveEventLoop) -> Vec<MonitorInfo> {
        event_loop.available_monitors().map(|monitor| {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            
            MonitorInfo {
                name: monitor.name().unwrap_or_default(),
                physical_width: size.width,
                physical_height: size.height,
                logical_width: size.width as f64 / scale,
                logical_height: size.height as f64 / scale,
                scale_factor: scale,
                refresh_rate: monitor.refresh_rate_millihertz().map(|r| r / 1000),
            }
        }).collect()
    }

    pub fn suggest_resolution(monitor: &MonitorInfo, target_aspect: f32) -> (u32, u32) {
        let monitor_aspect = monitor.logical_width as f32 / monitor.logical_height as f32;
        
        if (monitor_aspect - target_aspect).abs() < 0.1 {
            // Same aspect ratio, use 80% of screen
            let w = (monitor.logical_width * 0.8) as u32;
            let h = (monitor.logical_height * 0.8) as u32;
            (w, h)
        } else if target_aspect > monitor_aspect {
            // Target is wider
            let w = (monitor.logical_width * 0.9) as u32;
            let h = (w as f32 / target_aspect) as u32;
            (w, h)
        } else {
            // Target is taller
            let h = (monitor.logical_height * 0.9) as u32;
            let w = (h as f32 * target_aspect) as u32;
            (w, h)
        }
    }
}
