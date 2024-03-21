use aws_sdk_cloudwatchlogs::types::ResultField;
use tabled::Tabled;

#[derive(Debug, Clone, Default, Tabled)]
pub struct Stats {
    pub mem: u32,
    pub count: u32,
    pub stddev: f32,
    pub min: f32,
    pub p50: f32,
    pub p75: f32,
    pub p99: f32,
    pub p995: f32,
    pub p999: f32,
    pub max: f32,
}

impl Stats {
    pub fn update(&mut self, result: &ResultField) {
        match result.field().unwrap().trim() {
            "memorySize" => {
                self.mem = result.value().unwrap().parse().unwrap();
            }
            "count" => {
                self.count = result.value().unwrap().parse().unwrap();
            }
            "stddev" => {
                self.stddev = result.value().unwrap().parse().unwrap();
            }
            "min" => {
                self.min = result.value().unwrap().parse().unwrap();
            }
            "max" => {
                self.max = result.value().unwrap().parse().unwrap();
            }
            "p50" => {
                self.p50 = result.value().unwrap().parse().unwrap();
            }
            "p75" => {
                self.p75 = result.value().unwrap().parse().unwrap();
            }
            "p99" => {
                self.p99 = result.value().unwrap().parse().unwrap();
            }
            "p995" => {
                self.p995 = result.value().unwrap().parse().unwrap();
            }
            "p999" => {
                self.p999 = result.value().unwrap().parse().unwrap();
            }
            _ => {}
        }
    }
}
