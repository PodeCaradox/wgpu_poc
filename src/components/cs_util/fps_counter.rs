use std::collections::VecDeque;
use std::time::Duration;

use instant::Instant;
use log::{error, warn};

#[derive(Debug)]
pub struct FPSCounter {
    pub frames_per_second: i32,
    pub updates_per_second: i32,
    time_per_frame: f64,
    time_per_update: f64,
    one_second: Duration,
    timer: Duration,
    frames_counter: i32,
    update_counter: i32,
}

impl FPSCounter {
    pub(crate) fn new() -> Self {
        Self {
            frames_per_second: 0,
            updates_per_second: 0,
            time_per_frame: 0.0,
            time_per_update: 0.0,
            one_second: Duration::from_secs(1),
            timer: Duration::new(0, 0),
            frames_counter: 0,
            update_counter: 0,
        }
    }

    pub fn tick(&mut self, elapsed_time: Duration) {
        self.update_counter += 1;
        self.frames_counter += 1;
        self.timer += elapsed_time;

        if self.timer <= self.one_second {
            return;
        }

        self.updates_per_second = self.update_counter;
        self.frames_per_second = self.frames_counter;
        self.update_counter = 0;
        self.frames_counter = 0;
        self.timer -= self.one_second;
        error!("FPS: {:?}",self.frames_per_second);
    }
}