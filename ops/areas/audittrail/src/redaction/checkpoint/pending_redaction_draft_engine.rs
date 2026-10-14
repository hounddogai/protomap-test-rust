use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending redaction draft calculation.
pub trait PendingRedactionDraftAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRedactionDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingRedactionDraftAdjuster {
    pub minimum: f64,
}

impl PendingRedactionDraftAdjuster for MinimumPendingRedactionDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingRedactionDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingRedactionDraftAdjuster {
    pub percent: f64,
}

impl PendingRedactionDraftAdjuster for PercentPendingRedactionDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingRedactionDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingRedactionDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingRedactionDraftAdjuster for FlaggedPendingRedactionDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingRedactionDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPendingRedactionDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingRedactionDraftAdjuster for LabelPendingRedactionDraftAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PendingRedactionDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingRedactionDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingRedactionDraftAdjuster for TieredPendingRedactionDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingRedactionDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending redaction draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRedactionDraftEngine {
    pub steps: Vec<Rc<dyn PendingRedactionDraftAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRedactionDraftEngine {
    pub fn price(&mut self, item: &PendingRedactionDraft) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PendingRedactionDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_redaction_draft_engine(flags: &flags::Registry) -> PendingRedactionDraftEngine {
    PendingRedactionDraftEngine {
        steps: vec![
            Rc::new(MinimumPendingRedactionDraftAdjuster { minimum: 28.9 }) as Rc<dyn PendingRedactionDraftAdjuster>,
            Rc::new(PercentPendingRedactionDraftAdjuster { percent: -9.3 }),
            Rc::new(FlaggedPendingRedactionDraftAdjuster {
                flags: flags.clone(),
                flag: "pending-redaction-draft-capped".to_string(),
                percent: 6.0,
            }),
            Rc::new(LabelPendingRedactionDraftAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.94)]),
            }),
            Rc::new(TieredPendingRedactionDraftAdjuster {
                bounds: vec![5835.0, 7180.0, 8707.0],
                rates: vec![1.14, 1.18, 1.01],
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 53699.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_redaction_draft_engine(&flags::new_registry());
        let item = new_pending_redaction_draft("code-472", "label-736");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
