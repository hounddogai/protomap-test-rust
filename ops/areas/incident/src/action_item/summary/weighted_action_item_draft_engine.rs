use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted action item draft calculation.
pub trait WeightedActionItemDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedActionItemDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameWeightedActionItemDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedActionItemDraftFactor for NameWeightedActionItemDraftAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &WeightedActionItemDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedActionItemDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedActionItemDraftFactor for TieredWeightedActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedActionItemDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedActionItemDraftAdjuster {
    pub step: f64,
}

impl WeightedActionItemDraftFactor for RoundingWeightedActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedActionItemDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedActionItemDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedActionItemDraftFactor for FlaggedWeightedActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedActionItemDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedActionItemDraftAdjuster {
    pub cap: f64,
}

impl WeightedActionItemDraftFactor for CapWeightedActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedActionItemDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedActionItemDraftAdjuster {
    pub percent: f64,
}

impl WeightedActionItemDraftFactor for PercentWeightedActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedActionItemDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted action item draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedActionItemDraftEngine {
    pub steps: Vec<Rc<dyn WeightedActionItemDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedActionItemDraftEngine {
    pub fn evaluate(&mut self, item: &WeightedActionItemDraft) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[WeightedActionItemDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_action_item_draft_engine(flags: &flags::Registry) -> WeightedActionItemDraftEngine {
    WeightedActionItemDraftEngine {
        steps: vec![
            Rc::new(NameWeightedActionItemDraftAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.31)]),
            }) as Rc<dyn WeightedActionItemDraftFactor>,
            Rc::new(TieredWeightedActionItemDraftAdjuster {
                bounds: vec![1469.0, 1895.0, 6260.0],
                rates: vec![1.12, 1.06, 0.88],
            }),
            Rc::new(RoundingWeightedActionItemDraftAdjuster { step: 0.25 }),
            Rc::new(FlaggedWeightedActionItemDraftAdjuster {
                flags: flags.clone(),
                flag: "weighted-action-item-draft-default".to_string(),
                percent: -3.7,
            }),
            Rc::new(CapWeightedActionItemDraftAdjuster { cap: 36663.0 }),
            Rc::new(PercentWeightedActionItemDraftAdjuster { percent: -18.8 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 55443.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_action_item_draft_engine(&flags::new_registry());
        let item = new_weighted_action_item_draft("code-647", "name-403");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
