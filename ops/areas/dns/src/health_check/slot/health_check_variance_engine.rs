use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check variance calculation.
pub trait HealthCheckVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckVarianceAdjuster {
    pub percent: f64,
}

impl HealthCheckVarianceStep for PercentHealthCheckVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapHealthCheckVarianceAdjuster {
    pub cap: f64,
}

impl HealthCheckVarianceStep for CapHealthCheckVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &HealthCheckVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckVarianceStep for FlaggedHealthCheckVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the health check variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckVarianceEngine {
    pub steps: Vec<Rc<dyn HealthCheckVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckVarianceEngine {
    pub fn run(&mut self, item: &HealthCheckVariance) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_health_check_variance_engine(flags: &flags::Registry) -> HealthCheckVarianceEngine {
    HealthCheckVarianceEngine {
        steps: vec![
            Rc::new(PercentHealthCheckVarianceAdjuster { percent: -17.5 }) as Rc<dyn HealthCheckVarianceStep>,
            Rc::new(CapHealthCheckVarianceAdjuster { cap: 1638.0 }),
            Rc::new(FlaggedHealthCheckVarianceAdjuster {
                flags: flags.clone(),
                flag: "health-check-variance-scheduled".to_string(),
                percent: -7.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 27041.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_variance_engine(&flags::new_registry());
        let item = new_health_check_variance("code-177", "title-165");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
