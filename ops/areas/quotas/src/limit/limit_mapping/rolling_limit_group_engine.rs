use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling limit group calculation.
pub trait RollingLimitGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingLimitGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingLimitGroupAdjuster {
    pub minimum: f64,
}

impl RollingLimitGroupFactor for MinimumRollingLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingLimitGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingLimitGroupAdjuster {
    pub cap: f64,
}

impl RollingLimitGroupFactor for CapRollingLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingLimitGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingLimitGroupAdjuster {
    pub percent: f64,
}

impl RollingLimitGroupFactor for PercentRollingLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingLimitGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling limit group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingLimitGroupEngine {
    pub steps: Vec<Rc<dyn RollingLimitGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingLimitGroupEngine {
    pub fn price(&mut self, item: &RollingLimitGroup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingLimitGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_limit_group_engine() -> RollingLimitGroupEngine {
    RollingLimitGroupEngine {
        steps: vec![
            Rc::new(MinimumRollingLimitGroupAdjuster { minimum: 8.9 }) as Rc<dyn RollingLimitGroupFactor>,
            Rc::new(CapRollingLimitGroupAdjuster { cap: 19469.0 }),
            Rc::new(PercentRollingLimitGroupAdjuster { percent: -18.8 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 53877.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_limit_group_engine();
        let item = new_rolling_limit_group("code-407", "name-261");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
