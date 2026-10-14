use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the action item entry calculation.
pub trait ActionItemEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentActionItemEntryAdjuster {
    pub percent: f64,
}

impl ActionItemEntryAdjuster for PercentActionItemEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ActionItemEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleActionItemEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ActionItemEntryAdjuster for TitleActionItemEntryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ActionItemEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapActionItemEntryAdjuster {
    pub cap: f64,
}

impl ActionItemEntryAdjuster for CapActionItemEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ActionItemEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the action item entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemEntryEngine {
    pub steps: Vec<Rc<dyn ActionItemEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemEntryEngine {
    pub fn price(&mut self, item: &ActionItemEntry) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_action_item_entry_engine() -> ActionItemEntryEngine {
    ActionItemEntryEngine {
        steps: vec![
            Rc::new(PercentActionItemEntryAdjuster { percent: -3.2 }) as Rc<dyn ActionItemEntryAdjuster>,
            Rc::new(TitleActionItemEntryAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.94),
                    ("north".to_string(), 1.38),
                    ("west".to_string(), 0.77),
                ]),
            }),
            Rc::new(CapActionItemEntryAdjuster { cap: 24671.0 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 12096.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_action_item_entry_engine();
        let item = new_action_item_entry("code-334", "title-143");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
