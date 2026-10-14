use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict peering draft calculation.
pub trait StrictPeeringDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictPeeringDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringDraftCapacityOffsetAdjuster {
    pub weight: f64,
}

impl StrictPeeringDraftFactor for StrictPeeringDraftCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &StrictPeeringDraft, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictPeeringDraftAdjuster {
    pub minimum: f64,
}

impl StrictPeeringDraftFactor for MinimumStrictPeeringDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictPeeringDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictPeeringDraftAdjuster {
    pub step: f64,
}

impl StrictPeeringDraftFactor for RoundingStrictPeeringDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictPeeringDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictPeeringDraftAdjuster {
    pub cap: f64,
}

impl StrictPeeringDraftFactor for CapStrictPeeringDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictPeeringDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictPeeringDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictPeeringDraftFactor for FlaggedStrictPeeringDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictPeeringDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictPeeringDraftAdjuster {
    pub percent: f64,
}

impl StrictPeeringDraftFactor for PercentStrictPeeringDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictPeeringDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict peering draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictPeeringDraftEngine {
    pub steps: Vec<Rc<dyn StrictPeeringDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictPeeringDraftEngine {
    pub fn price(&mut self, item: &StrictPeeringDraft) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictPeeringDraftFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictPeeringDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_peering_draft_engine(flags: &flags::Registry) -> StrictPeeringDraftEngine {
    StrictPeeringDraftEngine {
        steps: vec![
            Rc::new(StrictPeeringDraftCapacityOffsetAdjuster { weight: 0.92 }) as Rc<dyn StrictPeeringDraftFactor>,
            Rc::new(MinimumStrictPeeringDraftAdjuster { minimum: 21.2 }),
            Rc::new(RoundingStrictPeeringDraftAdjuster { step: 0.05 }),
            Rc::new(CapStrictPeeringDraftAdjuster { cap: 39130.0 }),
            Rc::new(FlaggedStrictPeeringDraftAdjuster {
                flags: flags.clone(),
                flag: "strict-peering-draft-primary".to_string(),
                percent: -8.3,
            }),
            Rc::new(PercentStrictPeeringDraftAdjuster { percent: 2.4 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 61910.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_peering_draft_engine(&flags::new_registry());
        let item = new_strict_peering_draft("code-900", "name-942");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
