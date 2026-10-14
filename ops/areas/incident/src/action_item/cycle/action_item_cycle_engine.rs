use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the action item cycle calculation.
pub trait ActionItemCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemCycleStep for TieredActionItemCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemCycleDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl ActionItemCycleStep for ActionItemCycleDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &ActionItemCycle, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedActionItemCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ActionItemCycleStep for FlaggedActionItemCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ActionItemCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the action item cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemCycleEngine {
    pub steps: Vec<Rc<dyn ActionItemCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemCycleEngine {
    pub fn evaluate(&mut self, item: &ActionItemCycle) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ActionItemCycleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_action_item_cycle_engine(flags: &flags::Registry) -> ActionItemCycleEngine {
    ActionItemCycleEngine {
        steps: vec![
            Rc::new(TieredActionItemCycleAdjuster {
                bounds: vec![302.0, 401.0, 4324.0, 9074.0],
                rates: vec![1.09, 1.13, 0.83, 0.84],
            }) as Rc<dyn ActionItemCycleStep>,
            Rc::new(ActionItemCycleDurationMinutesOffsetAdjuster { weight: 1.53 }),
            Rc::new(FlaggedActionItemCycleAdjuster {
                flags: flags.clone(),
                flag: "action-item-cycle-batched".to_string(),
                percent: -0.4,
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 58338.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_action_item_cycle_engine(&flags::new_registry());
        let item = new_action_item_cycle("code-49", "name-608");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
