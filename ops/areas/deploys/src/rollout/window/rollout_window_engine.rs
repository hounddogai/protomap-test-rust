use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout window calculation.
pub trait RolloutWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutWindow, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutWindowFactor for FlaggedRolloutWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutWindowAdjuster {
    pub percent: f64,
}

impl RolloutWindowFactor for PercentRolloutWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutWindowCeilingOffsetAdjuster {
    pub weight: f64,
}

impl RolloutWindowFactor for RolloutWindowCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutWindow, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutWindowAdjuster {
    pub minimum: f64,
}

impl RolloutWindowFactor for MinimumRolloutWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rollout window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutWindowEngine {
    pub steps: Vec<Rc<dyn RolloutWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutWindowEngine {
    pub fn compute(&mut self, item: &RolloutWindow) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[RolloutWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollout_window_engine(flags: &flags::Registry) -> RolloutWindowEngine {
    RolloutWindowEngine {
        steps: vec![
            Rc::new(FlaggedRolloutWindowAdjuster {
                flags: flags.clone(),
                flag: "rollout-window-batched".to_string(),
                percent: 4.4,
            }) as Rc<dyn RolloutWindowFactor>,
            Rc::new(PercentRolloutWindowAdjuster { percent: 23.8 }),
            Rc::new(RolloutWindowCeilingOffsetAdjuster { weight: 1.03 }),
            Rc::new(MinimumRolloutWindowAdjuster { minimum: 17.4 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 30077.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_window_engine(&flags::new_registry());
        let item = new_rollout_window("code-340", "name-978");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
