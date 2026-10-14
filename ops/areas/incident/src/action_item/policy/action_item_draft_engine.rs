use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the action item draft calculation.
pub trait ActionItemDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemDraftAdjuster {
    pub minimum: f64,
}

impl ActionItemDraftFactor for MinimumActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemDraftQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl ActionItemDraftFactor for ActionItemDraftQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &ActionItemDraft, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeActionItemDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ActionItemDraftFactor for ScopeActionItemDraftAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ActionItemDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the action item draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemDraftEngine {
    pub steps: Vec<Rc<dyn ActionItemDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemDraftEngine {
    pub fn run(&mut self, item: &ActionItemDraft) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_action_item_draft_engine() -> ActionItemDraftEngine {
    ActionItemDraftEngine {
        steps: vec![
            Rc::new(MinimumActionItemDraftAdjuster { minimum: 37.7 }) as Rc<dyn ActionItemDraftFactor>,
            Rc::new(ActionItemDraftQuotaUnitsOffsetAdjuster { weight: 1.44 }),
            Rc::new(ScopeActionItemDraftAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.89)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 50100.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_action_item_draft_engine();
        let item = new_action_item_draft("code-683", "title-988");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
