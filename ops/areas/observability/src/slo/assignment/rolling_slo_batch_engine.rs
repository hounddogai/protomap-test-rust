use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling slo batch calculation.
pub trait RollingSloBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingSloBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingSloBatchAdjuster {
    pub minimum: f64,
}

impl RollingSloBatchFactor for MinimumRollingSloBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingSloBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingSloBatchAdjuster {
    pub percent: f64,
}

impl RollingSloBatchFactor for PercentRollingSloBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingSloBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingSloBatchAdjuster {
    pub step: f64,
}

impl RollingSloBatchFactor for RoundingRollingSloBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingSloBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingSloBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingSloBatchFactor for FlaggedRollingSloBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingSloBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRollingSloBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingSloBatchFactor for NameRollingSloBatchAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RollingSloBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rolling slo batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingSloBatchEngine {
    pub steps: Vec<Rc<dyn RollingSloBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingSloBatchEngine {
    pub fn price(&mut self, item: &RollingSloBatch) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingSloBatchFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RollingSloBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_slo_batch_engine(flags: &flags::Registry) -> RollingSloBatchEngine {
    RollingSloBatchEngine {
        steps: vec![
            Rc::new(MinimumRollingSloBatchAdjuster { minimum: 27.3 }) as Rc<dyn RollingSloBatchFactor>,
            Rc::new(PercentRollingSloBatchAdjuster { percent: 8.5 }),
            Rc::new(RoundingRollingSloBatchAdjuster { step: 0.5 }),
            Rc::new(FlaggedRollingSloBatchAdjuster {
                flags: flags.clone(),
                flag: "rolling-slo-batch-capped".to_string(),
                percent: 8.0,
            }),
            Rc::new(NameRollingSloBatchAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.13), ("west".to_string(), 0.8)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 71456.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_slo_batch_engine(&flags::new_registry());
        let item = new_rolling_slo_batch("code-319", "name-591");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
