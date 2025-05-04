use std::time::Instant;

pub struct Time {
    pub(crate) last_frame_time: Instant,
}

impl Time {
    pub(crate) fn new() -> Self {
        Self {
            last_frame_time: Instant::now(),
        }
    }

    pub fn delta_time(&self) -> f32 {
        Instant::now()
            .duration_since(self.last_frame_time)
            .as_secs_f32()
    }
}
