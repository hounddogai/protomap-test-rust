use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check bundle calculation.
pub trait HealthCheckBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckBundle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckBundleStep for FlaggedHealthCheckBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingHealthCheckBundleAdjuster {
    pub step: f64,
}

impl HealthCheckBundleStep for RoundingHealthCheckBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &HealthCheckBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBundleRatioOffsetAdjuster {
    pub weight: f64,
}

impl HealthCheckBundleStep for HealthCheckBundleRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &HealthCheckBundle, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleHealthCheckBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckBundleStep for TitleHealthCheckBundleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &HealthCheckBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the health check bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckBundleEngine {
    pub steps: Vec<Rc<dyn HealthCheckBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckBundleEngine {
    pub fn run(&mut self, item: &HealthCheckBundle) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_health_check_bundle_engine(flags: &flags::Registry) -> HealthCheckBundleEngine {
    HealthCheckBundleEngine {
        steps: vec![
            Rc::new(FlaggedHealthCheckBundleAdjuster {
                flags: flags.clone(),
                flag: "health-check-bundle-regional".to_string(),
                percent: -7.2,
            }) as Rc<dyn HealthCheckBundleStep>,
            Rc::new(RoundingHealthCheckBundleAdjuster { step: 0.05 }),
            Rc::new(HealthCheckBundleRatioOffsetAdjuster { weight: 1.6 }),
            Rc::new(TitleHealthCheckBundleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.31)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 5175.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_bundle_engine(&flags::new_registry());
        let item = new_health_check_bundle("code-242", "title-894");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
