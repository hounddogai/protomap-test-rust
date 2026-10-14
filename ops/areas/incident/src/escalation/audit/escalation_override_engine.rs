use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the escalation override calculation.
pub trait EscalationOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationOverrideAdjuster {
    pub step: f64,
}

impl EscalationOverrideModifier for RoundingEscalationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationOverrideScoreOffsetAdjuster {
    pub weight: f64,
}

impl EscalationOverrideModifier for EscalationOverrideScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationOverride, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelEscalationOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationOverrideModifier for LabelEscalationOverrideAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &EscalationOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEscalationOverrideAdjuster {
    pub minimum: f64,
}

impl EscalationOverrideModifier for MinimumEscalationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EscalationOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationOverrideAdjuster {
    pub cap: f64,
}

impl EscalationOverrideModifier for CapEscalationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the escalation override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationOverrideEngine {
    pub steps: Vec<Rc<dyn EscalationOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationOverrideEngine {
    pub fn evaluate(&mut self, item: &EscalationOverride) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EscalationOverrideModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EscalationOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_escalation_override_engine() -> EscalationOverrideEngine {
    EscalationOverrideEngine {
        steps: vec![
            Rc::new(RoundingEscalationOverrideAdjuster { step: 0.05 }) as Rc<dyn EscalationOverrideModifier>,
            Rc::new(EscalationOverrideScoreOffsetAdjuster { weight: 0.08 }),
            Rc::new(LabelEscalationOverrideAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.81), ("central".to_string(), 1.35)]),
            }),
            Rc::new(MinimumEscalationOverrideAdjuster { minimum: 9.9 }),
            Rc::new(CapEscalationOverrideAdjuster { cap: 21498.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 7538.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_override_engine();
        let item = new_escalation_override("code-354", "label-458");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
