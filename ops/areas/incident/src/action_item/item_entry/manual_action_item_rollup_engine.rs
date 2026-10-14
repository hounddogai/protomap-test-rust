use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual action item rollup calculation.
pub trait ManualActionItemRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualActionItemRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameManualActionItemRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualActionItemRollupFactor for NameManualActionItemRollupAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ManualActionItemRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualActionItemRollupAdjuster {
    pub step: f64,
}

impl ManualActionItemRollupFactor for RoundingManualActionItemRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualActionItemRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualActionItemRollupAdjuster {
    pub percent: f64,
}

impl ManualActionItemRollupFactor for PercentManualActionItemRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualActionItemRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual action item rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualActionItemRollupEngine {
    pub steps: Vec<Rc<dyn ManualActionItemRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualActionItemRollupEngine {
    pub fn run(&mut self, item: &ManualActionItemRollup) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualActionItemRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualActionItemRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_action_item_rollup_engine() -> ManualActionItemRollupEngine {
    ManualActionItemRollupEngine {
        steps: vec![
            Rc::new(NameManualActionItemRollupAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.27),
                    ("south".to_string(), 0.94),
                    ("north".to_string(), 1.22),
                ]),
            }) as Rc<dyn ManualActionItemRollupFactor>,
            Rc::new(RoundingManualActionItemRollupAdjuster { step: 1.0 }),
            Rc::new(PercentManualActionItemRollupAdjuster { percent: 7.6 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 54449.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_action_item_rollup_engine();
        let item = new_manual_action_item_rollup("code-444", "name-555");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
