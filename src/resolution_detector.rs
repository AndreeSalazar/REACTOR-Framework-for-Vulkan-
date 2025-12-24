use winit::event_loop::ActiveEventLoop;

pub struct ResolutionDetector;

impl ResolutionDetector {
    /// Detects the best resolution for the primary monitor, trying to match the target resolution
    /// but ensuring it fits within the monitor's logical size.
    pub fn get_smart_resolution(
        event_loop: &ActiveEventLoop,
        target_width: f32,
        target_height: f32,
    ) -> (f64, f64) {
        let monitor = event_loop.primary_monitor().or_else(|| event_loop.available_monitors().next());
        
        if let Some(monitor) = monitor {
            let size = monitor.size();
            let scale = monitor.scale_factor(); // dpi scale
            
            // Convert physical size to logical size
            let mon_width_logical = size.width as f64 / scale;
            let mon_height_logical = size.height as f64 / scale;
            
            println!("Detected Monitor: {} ({:?})", monitor.name().unwrap_or_default(), size);
            println!("  Scale Factor: {}", scale);
            println!("  Logical Size: {}x{}", mon_width_logical, mon_height_logical);

            // Respect screen resolution: Don't exceed monitor size
            // We take the minimum of target and available space
            let w = if (target_width as f64) > mon_width_logical { mon_width_logical } else { target_width as f64 };
            let h = if (target_height as f64) > mon_height_logical { mon_height_logical } else { target_height as f64 };
            
            println!("  Selected Resolution: {}x{}", w, h);
            (w, h)
        } else {
            println!("No monitor detected, using default target.");
            (target_width as f64, target_height as f64)
        }
    }
}
