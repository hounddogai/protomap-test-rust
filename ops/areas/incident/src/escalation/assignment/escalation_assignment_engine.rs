use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the escalation assignment calculation.
pub trait EscalationAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationAssignmentAdjuster for TieredEscalationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationAssignmentAdjuster {
    pub cap: f64,
}

impl EscalationAssignmentAdjuster for CapEscalationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEscalationAssignmentAdjuster {
    pub minimum: f64,
}

impl EscalationAssignmentAdjuster for MinimumEscalationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EscalationAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the escalation assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationAssignmentEngine {
    pub steps: Vec<Rc<dyn EscalationAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationAssignmentEngine {
    pub fn price(&mut self, item: &EscalationAssignment) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[EscalationAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_escalation_assignment_engine() -> EscalationAssignmentEngine {
    EscalationAssignmentEngine {
        steps: vec![
            Rc::new(TieredEscalationAssignmentAdjuster {
                bounds: vec![2613.0, 3101.0, 6967.0, 9475.0],
                rates: vec![0.95, 0.86, 1.09, 1.13],
            }) as Rc<dyn EscalationAssignmentAdjuster>,
            Rc::new(CapEscalationAssignmentAdjuster { cap: 10404.0 }),
            Rc::new(MinimumEscalationAssignmentAdjuster { minimum: 29.4 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 81848.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_assignment_engine();
        let item = new_escalation_assignment("code-398", "title-873");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
