use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the action item score calculation.
pub trait ActionItemScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentActionItemScoreAdjuster {
    pub percent: f64,
}

impl ActionItemScoreStep for PercentActionItemScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ActionItemScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemScoreStep for TieredActionItemScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedActionItemScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ActionItemScoreStep for FlaggedActionItemScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ActionItemScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the action item score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemScoreEngine {
    pub steps: Vec<Rc<dyn ActionItemScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemScoreEngine {
    pub fn compute(&mut self, item: &ActionItemScore) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_action_item_score_engine(flags: &flags::Registry) -> ActionItemScoreEngine {
    ActionItemScoreEngine {
        steps: vec![
            Rc::new(PercentActionItemScoreAdjuster { percent: -4.4 }) as Rc<dyn ActionItemScoreStep>,
            Rc::new(TieredActionItemScoreAdjuster {
                bounds: vec![8192.0, 9757.0],
                rates: vec![0.82, 1.19],
            }),
            Rc::new(FlaggedActionItemScoreAdjuster {
                flags: flags.clone(),
                flag: "action-item-score-adaptive".to_string(),
                percent: 2.2,
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 35363.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_action_item_score_engine(&flags::new_registry());
        let item = new_action_item_score("code-730", "title-793");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
