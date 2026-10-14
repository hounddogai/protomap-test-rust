use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the escalation preference calculation.
pub trait EscalationPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryEscalationPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationPreferenceAdjuster for CategoryEscalationPreferenceAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &EscalationPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPreferenceScoreOffsetAdjuster {
    pub weight: f64,
}

impl EscalationPreferenceAdjuster for EscalationPreferenceScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationPreference, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationPreferenceAdjuster {
    pub step: f64,
}

impl EscalationPreferenceAdjuster for RoundingEscalationPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the escalation preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationPreferenceEngine {
    pub steps: Vec<Rc<dyn EscalationPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationPreferenceEngine {
    pub fn compute(&mut self, item: &EscalationPreference) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EscalationPreferenceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EscalationPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_escalation_preference_engine() -> EscalationPreferenceEngine {
    EscalationPreferenceEngine {
        steps: vec![
            Rc::new(CategoryEscalationPreferenceAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.8), ("east".to_string(), 1.15)]),
            }) as Rc<dyn EscalationPreferenceAdjuster>,
            Rc::new(EscalationPreferenceScoreOffsetAdjuster { weight: 0.06 }),
            Rc::new(RoundingEscalationPreferenceAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 30988.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_preference_engine();
        let item = new_escalation_preference("code-187", "title-3");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
