use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict action item draft calculation.
pub trait StrictActionItemDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictActionItemDraft, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictActionItemDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictActionItemDraftModifier for FlaggedStrictActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictActionItemDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictActionItemDraftAdjuster {
    pub percent: f64,
}

impl StrictActionItemDraftModifier for PercentStrictActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictActionItemDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictActionItemDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictActionItemDraftModifier for TieredStrictActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictActionItemDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictActionItemDraftAdjuster {
    pub step: f64,
}

impl StrictActionItemDraftModifier for RoundingStrictActionItemDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictActionItemDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictActionItemDraftUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl StrictActionItemDraftModifier for StrictActionItemDraftUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &StrictActionItemDraft, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

/// Runs the strict action item draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictActionItemDraftEngine {
    pub steps: Vec<Rc<dyn StrictActionItemDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictActionItemDraftEngine {
    pub fn compute(&mut self, item: &StrictActionItemDraft) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[StrictActionItemDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_action_item_draft_engine(flags: &flags::Registry) -> StrictActionItemDraftEngine {
    StrictActionItemDraftEngine {
        steps: vec![
            Rc::new(FlaggedStrictActionItemDraftAdjuster {
                flags: flags.clone(),
                flag: "strict-action-item-draft-rolling".to_string(),
                percent: 5.5,
            }) as Rc<dyn StrictActionItemDraftModifier>,
            Rc::new(PercentStrictActionItemDraftAdjuster { percent: 13.1 }),
            Rc::new(TieredStrictActionItemDraftAdjuster {
                bounds: vec![4114.0, 8313.0, 8974.0],
                rates: vec![0.85, 0.91, 1.2],
            }),
            Rc::new(RoundingStrictActionItemDraftAdjuster { step: 0.1 }),
            Rc::new(StrictActionItemDraftUpdatedAtOffsetAdjuster { weight: 0.73 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 92594.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_action_item_draft_engine(&flags::new_registry());
        let item = new_strict_action_item_draft("code-424", "title-59");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
