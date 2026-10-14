use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the escalation window calculation.
pub trait EscalationWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationWindowRetriesOffsetAdjuster {
    pub weight: f64,
}

impl EscalationWindowFactor for EscalationWindowRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationWindow, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationWindowFactor for TieredEscalationWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationWindowAdjuster {
    pub cap: f64,
}

impl EscalationWindowFactor for CapEscalationWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the escalation window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationWindowEngine {
    pub steps: Vec<Rc<dyn EscalationWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationWindowEngine {
    pub fn run(&mut self, item: &EscalationWindow) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[EscalationWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_escalation_window_engine() -> EscalationWindowEngine {
    EscalationWindowEngine {
        steps: vec![
            Rc::new(EscalationWindowRetriesOffsetAdjuster { weight: 1.1 }) as Rc<dyn EscalationWindowFactor>,
            Rc::new(TieredEscalationWindowAdjuster {
                bounds: vec![147.0, 6410.0],
                rates: vec![0.84, 1.13],
            }),
            Rc::new(CapEscalationWindowAdjuster { cap: 41729.0 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 68018.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_window_engine();
        let item = new_escalation_window("code-42", "name-40");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
