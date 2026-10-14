use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default audit event draft calculation.
pub trait DefaultAuditEventDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultAuditEventDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultAuditEventDraftAdjuster {
    pub percent: f64,
}

impl DefaultAuditEventDraftModifier for PercentDefaultAuditEventDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultAuditEventDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultAuditEventDraftAdjuster {
    pub minimum: f64,
}

impl DefaultAuditEventDraftModifier for MinimumDefaultAuditEventDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultAuditEventDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultAuditEventDraftAdjuster {
    pub step: f64,
}

impl DefaultAuditEventDraftModifier for RoundingDefaultAuditEventDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultAuditEventDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the default audit event draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultAuditEventDraftEngine {
    pub steps: Vec<Rc<dyn DefaultAuditEventDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultAuditEventDraftEngine {
    pub fn run(&mut self, item: &DefaultAuditEventDraft) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[DefaultAuditEventDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_audit_event_draft_engine() -> DefaultAuditEventDraftEngine {
    DefaultAuditEventDraftEngine {
        steps: vec![
            Rc::new(PercentDefaultAuditEventDraftAdjuster { percent: -3.2 }) as Rc<dyn DefaultAuditEventDraftModifier>,
            Rc::new(MinimumDefaultAuditEventDraftAdjuster { minimum: 1.2 }),
            Rc::new(RoundingDefaultAuditEventDraftAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 64640.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_audit_event_draft_engine();
        let item = new_default_audit_event_draft("code-748", "label-63");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
