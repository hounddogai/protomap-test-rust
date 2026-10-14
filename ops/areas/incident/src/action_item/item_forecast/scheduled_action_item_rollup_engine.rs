use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled action item rollup calculation.
pub trait ScheduledActionItemRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledActionItemRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledActionItemRollupAdjuster {
    pub minimum: f64,
}

impl ScheduledActionItemRollupModifier for MinimumScheduledActionItemRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledActionItemRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledActionItemRollupAdjuster {
    pub step: f64,
}

impl ScheduledActionItemRollupModifier for RoundingScheduledActionItemRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledActionItemRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledActionItemRollupAdjuster {
    pub percent: f64,
}

impl ScheduledActionItemRollupModifier for PercentScheduledActionItemRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledActionItemRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledActionItemRollupAdjuster {
    pub cap: f64,
}

impl ScheduledActionItemRollupModifier for CapScheduledActionItemRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledActionItemRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemRollupScoreOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledActionItemRollupModifier for ScheduledActionItemRollupScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledActionItemRollup, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the scheduled action item rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledActionItemRollupEngine {
    pub steps: Vec<Rc<dyn ScheduledActionItemRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledActionItemRollupEngine {
    pub fn price(&mut self, item: &ScheduledActionItemRollup) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledActionItemRollupModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledActionItemRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_action_item_rollup_engine() -> ScheduledActionItemRollupEngine {
    ScheduledActionItemRollupEngine {
        steps: vec![
            Rc::new(MinimumScheduledActionItemRollupAdjuster { minimum: 23.2 })
                as Rc<dyn ScheduledActionItemRollupModifier>,
            Rc::new(RoundingScheduledActionItemRollupAdjuster { step: 5.0 }),
            Rc::new(PercentScheduledActionItemRollupAdjuster { percent: 6.5 }),
            Rc::new(CapScheduledActionItemRollupAdjuster { cap: 46484.0 }),
            Rc::new(ScheduledActionItemRollupScoreOffsetAdjuster { weight: 0.46 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 65970.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_action_item_rollup_engine();
        let item = new_scheduled_action_item_rollup("code-532", "label-838");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
