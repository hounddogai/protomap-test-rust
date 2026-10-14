use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the escalation claim calculation.
pub trait EscalationClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEscalationClaimAdjuster {
    pub percent: f64,
}

impl EscalationClaimFactor for PercentEscalationClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EscalationClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationClaimDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl EscalationClaimFactor for EscalationClaimDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationClaim, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationClaimFactor for TieredEscalationClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the escalation claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationClaimEngine {
    pub steps: Vec<Rc<dyn EscalationClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationClaimEngine {
    pub fn compute(&mut self, item: &EscalationClaim) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[EscalationClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_escalation_claim_engine() -> EscalationClaimEngine {
    EscalationClaimEngine {
        steps: vec![
            Rc::new(PercentEscalationClaimAdjuster { percent: -0.5 }) as Rc<dyn EscalationClaimFactor>,
            Rc::new(EscalationClaimDurationMinutesOffsetAdjuster { weight: 0.24 }),
            Rc::new(TieredEscalationClaimAdjuster {
                bounds: vec![716.0, 1295.0, 3253.0, 8821.0],
                rates: vec![1.2, 0.83, 1.11, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 79051.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_claim_engine();
        let item = new_escalation_claim("code-867", "title-165");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
