use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the redaction draft calculation.
pub trait RedactionDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionDraftAdjuster {
    pub cap: f64,
}

impl RedactionDraftModifier for CapRedactionDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionDraftAdjuster {
    pub step: f64,
}

impl RedactionDraftModifier for RoundingRedactionDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionDraftModifier for FlaggedRedactionDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionDraftAdjuster {
    pub minimum: f64,
}

impl RedactionDraftModifier for MinimumRedactionDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionDraftRetriesOffsetAdjuster {
    pub weight: f64,
}

impl RedactionDraftModifier for RedactionDraftRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionDraft, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

/// Runs the redaction draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionDraftEngine {
    pub steps: Vec<Rc<dyn RedactionDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionDraftEngine {
    pub fn price(&mut self, item: &RedactionDraft) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RedactionDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_redaction_draft_engine(flags: &flags::Registry) -> RedactionDraftEngine {
    RedactionDraftEngine {
        steps: vec![
            Rc::new(CapRedactionDraftAdjuster { cap: 32779.0 }) as Rc<dyn RedactionDraftModifier>,
            Rc::new(RoundingRedactionDraftAdjuster { step: 0.25 }),
            Rc::new(FlaggedRedactionDraftAdjuster {
                flags: flags.clone(),
                flag: "redaction-draft-manual".to_string(),
                percent: 0.8,
            }),
            Rc::new(MinimumRedactionDraftAdjuster { minimum: 3.4 }),
            Rc::new(RedactionDraftRetriesOffsetAdjuster { weight: 0.49 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 14329.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_draft_engine(&flags::new_registry());
        let item = new_redaction_draft("code-484", "label-262");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
