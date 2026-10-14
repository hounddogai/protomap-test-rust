use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered approval draft calculation.
pub trait TieredApprovalDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredApprovalDraft, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredApprovalDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredApprovalDraftModifier for FlaggedTieredApprovalDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredApprovalDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredApprovalDraftAdjuster {
    pub step: f64,
}

impl TieredApprovalDraftModifier for RoundingTieredApprovalDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredApprovalDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalDraftRateOffsetAdjuster {
    pub weight: f64,
}

impl TieredApprovalDraftModifier for TieredApprovalDraftRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &TieredApprovalDraft, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

/// Runs the tiered approval draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredApprovalDraftEngine {
    pub steps: Vec<Rc<dyn TieredApprovalDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredApprovalDraftEngine {
    pub fn evaluate(&mut self, item: &TieredApprovalDraft) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredApprovalDraftModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredApprovalDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_approval_draft_engine(flags: &flags::Registry) -> TieredApprovalDraftEngine {
    TieredApprovalDraftEngine {
        steps: vec![
            Rc::new(FlaggedTieredApprovalDraftAdjuster {
                flags: flags.clone(),
                flag: "tiered-approval-draft-lenient".to_string(),
                percent: -1.4,
            }) as Rc<dyn TieredApprovalDraftModifier>,
            Rc::new(RoundingTieredApprovalDraftAdjuster { step: 0.1 }),
            Rc::new(TieredApprovalDraftRateOffsetAdjuster { weight: 1.16 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 67195.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_approval_draft_engine(&flags::new_registry());
        let item = new_tiered_approval_draft("code-987", "title-990");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
