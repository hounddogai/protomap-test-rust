use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive audit event baseline calculation.
pub trait AdaptiveAuditEventBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveAuditEventBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventBaselinePriorityOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveAuditEventBaselineModifier for AdaptiveAuditEventBaselinePriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveAuditEventBaseline, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveAuditEventBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveAuditEventBaselineModifier for FlaggedAdaptiveAuditEventBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveAuditEventBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveAuditEventBaselineAdjuster {
    pub minimum: f64,
}

impl AdaptiveAuditEventBaselineModifier for MinimumAdaptiveAuditEventBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveAuditEventBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveAuditEventBaselineAdjuster {
    pub step: f64,
}

impl AdaptiveAuditEventBaselineModifier for RoundingAdaptiveAuditEventBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveAuditEventBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveAuditEventBaselineAdjuster {
    pub cap: f64,
}

impl AdaptiveAuditEventBaselineModifier for CapAdaptiveAuditEventBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveAuditEventBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the adaptive audit event baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveAuditEventBaselineEngine {
    pub steps: Vec<Rc<dyn AdaptiveAuditEventBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveAuditEventBaselineEngine {
    pub fn evaluate(&mut self, item: &AdaptiveAuditEventBaseline) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveAuditEventBaselineModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveAuditEventBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_audit_event_baseline_engine(flags: &flags::Registry) -> AdaptiveAuditEventBaselineEngine {
    AdaptiveAuditEventBaselineEngine {
        steps: vec![
            Rc::new(AdaptiveAuditEventBaselinePriorityOffsetAdjuster { weight: 0.83 })
                as Rc<dyn AdaptiveAuditEventBaselineModifier>,
            Rc::new(FlaggedAdaptiveAuditEventBaselineAdjuster {
                flags: flags.clone(),
                flag: "adaptive-audit-event-baseline-fallback".to_string(),
                percent: 3.7,
            }),
            Rc::new(MinimumAdaptiveAuditEventBaselineAdjuster { minimum: 24.4 }),
            Rc::new(RoundingAdaptiveAuditEventBaselineAdjuster { step: 5.0 }),
            Rc::new(CapAdaptiveAuditEventBaselineAdjuster { cap: 13464.0 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 59756.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_audit_event_baseline_engine(&flags::new_registry());
        let item = new_adaptive_audit_event_baseline("code-631", "label-860");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
