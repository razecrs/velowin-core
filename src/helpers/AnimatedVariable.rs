use std::time::{Instant, Duration};

pub struct Bezier {
    pub p1: (f32, f32),
    pub p2: (f32, f32),
}

impl Bezier {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { p1: (x1, y1), p2: (x2, y2) }
    }

    pub fn evaluate(&self, t: f32) -> f32 {
        if t <= 0.0 { return 0.0; }
        if t >= 1.0 { return 1.0; }
        let cx = 3.0 * self.p1.0;
        let bx = 3.0 * (self.p2.0 - self.p1.0) - cx;
        let ax = 1.0 - cx - bx;
        let cy = 3.0 * self.p1.1;
        let by = 3.0 * (self.p2.1 - self.p1.1) - cy;
        let ay = 1.0 - cy - by;
        let sample_bezier_x = |t: f32| -> f32 { ((ax * t + bx) * t + cx) * t };
        let mut t_found = t;
        for _ in 0..8 {
            let x = sample_bezier_x(t_found) - t;
            if x.abs() < 1e-4 { break; }
            let d = (3.0 * ax * t_found + 2.0 * bx) * t_found + cx;
            if d.abs() < 1e-4 { break; }
            t_found -= x / d;
        }
        ((ay * t_found + by) * t_found + cy) * t_found
    }
}

pub struct AnimatedVariable<T> {
    pub value: T,
    pub target: T,
    pub start_value: T,
    pub start_time: Option<Instant>,
    pub duration: Duration,
    pub bezier: Bezier,
}

impl AnimatedVariable<f32> {
    pub fn new_f32(initial: f32) -> Self {
        Self {
            value: initial,
            target: initial,
            start_value: initial,
            start_time: None,
            duration: Duration::from_millis(300),
            bezier: Bezier::new(0.05, 0.9, 0.1, 1.05),
        }
    }

    pub fn set(&mut self, target: f32) {
        if (target - self.target).abs() < 1e-4 { return; }
        self.start_value = self.value;
        self.target = target;
        self.start_time = Some(Instant::now());
    }

    pub fn update(&mut self) -> bool {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs_f32();
            let total = self.duration.as_secs_f32();
            if elapsed >= total {
                self.value = self.target;
                self.start_time = None;
                return false;
            }
            let t = elapsed / total;
            let progress = self.bezier.evaluate(t);
            self.value = self.start_value + (self.target - self.start_value) * progress;
            return true;
        }
        false
    }
}

impl AnimatedVariable<i32> {
    pub fn new_i32(initial: i32) -> Self {
        Self {
            value: initial,
            target: initial,
            start_value: initial,
            start_time: None,
            duration: Duration::from_millis(300),
            bezier: Bezier::new(0.05, 0.9, 0.1, 1.05),
        }
    }

    pub fn set(&mut self, target: i32) {
        if target == self.target { return; }
        self.start_value = self.value;
        self.target = target;
        self.start_time = Some(Instant::now());
    }

    pub fn update(&mut self) -> bool {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs_f32();
            let total = self.duration.as_secs_f32();
            if elapsed >= total {
                self.value = self.target;
                self.start_time = None;
                return false;
            }
            let t = elapsed / total;
            let progress = self.bezier.evaluate(t);
            self.value = self.start_value + ((self.target - self.start_value) as f32 * progress) as i32;
            return true;
        }
        false
    }
}
