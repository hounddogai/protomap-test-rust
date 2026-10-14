use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default action item draft calculation.
pub trait DefaultActionItemDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultActionItemDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemDraftPriorityOffsetAdjuster {
    pub weight: f64,
}

impl DefaultActionItemDraftStep for DefaultActionItemDraftPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultActionItemDraft, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultActionItemDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultActionItemDraftStep for FlaggedDefaultActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultActionItemDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultActionItemDraftAdjuster {
    pub cap: f64,
}

impl DefaultActionItemDraftStep for CapDefaultActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultActionItemDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultActionItemDraftAdjuster {
    pub minimum: f64,
}

impl DefaultActionItemDraftStep for MinimumDefaultActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultActionItemDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultActionItemDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultActionItemDraftStep for TieredDefaultActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultActionItemDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultActionItemDraftAdjuster {
    pub percent: f64,
}

impl DefaultActionItemDraftStep for PercentDefaultActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultActionItemDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default action item draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultActionItemDraftEngine {
    pub steps: Vec<Rc<dyn DefaultActionItemDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultActionItemDraftEngine {
    pub fn compute(&mut self, item: &DefaultActionItemDraft) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultActionItemDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_action_item_draft_engine(flags: &flags::Registry) -> DefaultActionItemDraftEngine {
    DefaultActionItemDraftEngine {
        steps: vec![
            Rc::new(DefaultActionItemDraftPriorityOffsetAdjuster { weight: 1.08 })
                as Rc<dyn DefaultActionItemDraftStep>,
            Rc::new(FlaggedDefaultActionItemDraftAdjuster {
                flags: flags.clone(),
                flag: "default-action-item-draft-lenient".to_string(),
                percent: 1.3,
            }),
            Rc::new(CapDefaultActionItemDraftAdjuster { cap: 16503.0 }),
            Rc::new(MinimumDefaultActionItemDraftAdjuster { minimum: 47.3 }),
            Rc::new(TieredDefaultActionItemDraftAdjuster {
                bounds: vec![1921.0, 5518.0, 9644.0],
                rates: vec![0.84, 1.09, 0.93],
            }),
            Rc::new(PercentDefaultActionItemDraftAdjuster { percent: -10.8 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 25292.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_action_item_draft_engine(&flags::new_registry());
        let item = new_default_action_item_draft("code-778", "label-740");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
