use tracing::info;

pub struct ElapsedTimeTracing {
    label: String,
    start_time: std::time::Instant,
}

impl ElapsedTimeTracing {
    pub fn new(label: &str) -> Self {
        Self { 
            label: label.to_string(), 
            start_time: std::time::Instant::now() 
        }
    }

    pub fn trace(&self) {
        let elapsed = self.start_time.elapsed();
        let elapsed_ms = elapsed.as_millis();
        info!(time_ms = elapsed_ms, label = self.label, "{} took {} ms", self.label, elapsed_ms);
    }
}
