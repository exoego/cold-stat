use aws_sdk_cloudwatchlogs::types::ResultField;

#[derive(Debug, Copy, Clone)]
pub struct Stats {
    pub cold_starts: u32,
    pub min: f32,
    pub max: f32,
    pub p50: f32,
    pub p75: f32,
    pub p99: f32,
    pub p995: f32,
    pub p999: f32,
}

impl Stats {
    pub fn empty() -> Self {
        Self {
            cold_starts: 0,
            min: 0.0,
            max: 0.0,
            p50: 0.0,
            p75: 0.0,
            p99: 0.0,
            p995: 0.0,
            p999: 0.0,
        }
    }

    pub fn update(&mut self, result: &ResultField) -> () {
        let field = result.field().unwrap().trim();
        match &*field {
            "cold_starts" => { self.cold_starts = result.value().unwrap().parse().unwrap(); }
            "min" => { self.min = result.value().unwrap().parse().unwrap(); }
            "max" => { self.max = result.value().unwrap().parse().unwrap(); }
            "p50" => { self.p50 = result.value().unwrap().parse().unwrap(); }
            "p75" => { self.p75 = result.value().unwrap().parse().unwrap(); }
            "p99" => { self.p99 = result.value().unwrap().parse().unwrap(); }
            "p995" => { self.p995 = result.value().unwrap().parse().unwrap(); }
            "p999" => { self.p999 = result.value().unwrap().parse().unwrap(); }
            _ => {}
        }
    }
}
