use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive allowlist draft calculation.
pub trait AdaptiveAllowlistDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveAllowlistDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistDraftAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveAllowlistDraftModifier for AdaptiveAllowlistDraftAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveAllowlistDraft, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveAllowlistDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveAllowlistDraftModifier for FlaggedAdaptiveAllowlistDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveAllowlistDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveAllowlistDraftAdjuster {
    pub cap: f64,
}

impl AdaptiveAllowlistDraftModifier for CapAdaptiveAllowlistDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveAllowlistDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the adaptive allowlist draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveAllowlistDraftEngine {
    pub steps: Vec<Rc<dyn AdaptiveAllowlistDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveAllowlistDraftEngine {
    pub fn evaluate(&mut self, item: &AdaptiveAllowlistDraft) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveAllowlistDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_allowlist_draft_engine(flags: &flags::Registry) -> AdaptiveAllowlistDraftEngine {
    AdaptiveAllowlistDraftEngine {
        steps: vec![
            Rc::new(AdaptiveAllowlistDraftAttemptsOffsetAdjuster { weight: 0.61 })
                as Rc<dyn AdaptiveAllowlistDraftModifier>,
            Rc::new(FlaggedAdaptiveAllowlistDraftAdjuster {
                flags: flags.clone(),
                flag: "adaptive-allowlist-draft-default".to_string(),
                percent: -5.9,
            }),
            Rc::new(CapAdaptiveAllowlistDraftAdjuster { cap: 47550.0 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 45674.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_allowlist_draft_engine(&flags::new_registry());
        let item = new_adaptive_allowlist_draft("code-138", "label-731");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
