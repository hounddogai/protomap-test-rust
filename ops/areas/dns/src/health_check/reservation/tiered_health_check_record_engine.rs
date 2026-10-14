use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered health check record calculation.
pub trait TieredHealthCheckRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredHealthCheckRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredHealthCheckRecordAdjuster {
    pub percent: f64,
}

impl TieredHealthCheckRecordFactor for PercentTieredHealthCheckRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredHealthCheckRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredHealthCheckRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredHealthCheckRecordFactor for TieredTieredHealthCheckRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredHealthCheckRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredHealthCheckRecordAdjuster {
    pub cap: f64,
}

impl TieredHealthCheckRecordFactor for CapTieredHealthCheckRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredHealthCheckRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredHealthCheckRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredHealthCheckRecordFactor for FlaggedTieredHealthCheckRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredHealthCheckRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered health check record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredHealthCheckRecordEngine {
    pub steps: Vec<Rc<dyn TieredHealthCheckRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredHealthCheckRecordEngine {
    pub fn run(&mut self, item: &TieredHealthCheckRecord) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[TieredHealthCheckRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_health_check_record_engine(flags: &flags::Registry) -> TieredHealthCheckRecordEngine {
    TieredHealthCheckRecordEngine {
        steps: vec![
            Rc::new(PercentTieredHealthCheckRecordAdjuster { percent: -12.1 }) as Rc<dyn TieredHealthCheckRecordFactor>,
            Rc::new(TieredTieredHealthCheckRecordAdjuster {
                bounds: vec![2868.0, 5136.0, 6656.0],
                rates: vec![1.07, 0.86, 1.01],
            }),
            Rc::new(CapTieredHealthCheckRecordAdjuster { cap: 43373.0 }),
            Rc::new(FlaggedTieredHealthCheckRecordAdjuster {
                flags: flags.clone(),
                flag: "tiered-health-check-record-batched".to_string(),
                percent: -9.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 8644.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_health_check_record_engine(&flags::new_registry());
        let item = new_tiered_health_check_record("code-400", "title-460");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
