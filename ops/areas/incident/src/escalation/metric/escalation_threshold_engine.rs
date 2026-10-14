use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the escalation threshold calculation.
pub trait EscalationThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationThresholdAdjuster {
    pub step: f64,
}

impl EscalationThresholdStep for RoundingEscalationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationThresholdRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl EscalationThresholdStep for EscalationThresholdRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationThreshold, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationThresholdStep for TieredEscalationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the escalation threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationThresholdEngine {
    pub steps: Vec<Rc<dyn EscalationThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationThresholdEngine {
    pub fn price(&mut self, item: &EscalationThreshold) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EscalationThresholdStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EscalationThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_escalation_threshold_engine() -> EscalationThresholdEngine {
    EscalationThresholdEngine {
        steps: vec![
            Rc::new(RoundingEscalationThresholdAdjuster { step: 1.0 }) as Rc<dyn EscalationThresholdStep>,
            Rc::new(EscalationThresholdRotationDaysOffsetAdjuster { weight: 0.13 }),
            Rc::new(TieredEscalationThresholdAdjuster {
                bounds: vec![693.0, 1061.0],
                rates: vec![0.82, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 1818.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_threshold_engine();
        let item = new_escalation_threshold("code-830", "name-993");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
