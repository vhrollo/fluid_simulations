use std::time::{Duration, Instant};

pub struct FpsTracker {
    frame_times: Vec<Instant>
}

impl FpsTracker {
    pub fn new() -> Self {
        Self {
            frame_times: Vec::new()
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.frame_times.push(now);
    
        // Remove old frame times
        let cutoff = now - Duration::from_secs(2);
        self.frame_times.retain(|&t| t > cutoff); // operates in place, faster than filter
    }

    pub fn get_fps(&self) -> f32 {
        if self.frame_times.len() < 2 {
            return 0.0;
        }

        let duration = self.frame_times.last().unwrap()
            .duration_since(
                *self.frame_times.first().unwrap()
            ).as_secs_f32();
        if duration == 0.0 {
            return 0.0;
        }

        self.frame_times.len() as f32 / duration
    }
}