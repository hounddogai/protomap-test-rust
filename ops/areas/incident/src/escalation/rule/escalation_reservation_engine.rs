use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the escalation reservation calculation.
pub trait EscalationReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEscalationReservationAdjuster {
    pub minimum: f64,
}

impl EscalationReservationModifier for MinimumEscalationReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EscalationReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationReservationThresholdOffsetAdjuster {
    pub weight: f64,
}

impl EscalationReservationModifier for EscalationReservationThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationReservation, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelEscalationReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationReservationModifier for LabelEscalationReservationAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &EscalationReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationReservationModifier for TieredEscalationReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationReservationAdjuster {
    pub cap: f64,
}

impl EscalationReservationModifier for CapEscalationReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the escalation reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationReservationEngine {
    pub steps: Vec<Rc<dyn EscalationReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationReservationEngine {
    pub fn run(&mut self, item: &EscalationReservation) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[EscalationReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_escalation_reservation_engine() -> EscalationReservationEngine {
    EscalationReservationEngine {
        steps: vec![
            Rc::new(MinimumEscalationReservationAdjuster { minimum: 8.7 }) as Rc<dyn EscalationReservationModifier>,
            Rc::new(EscalationReservationThresholdOffsetAdjuster { weight: 0.44 }),
            Rc::new(LabelEscalationReservationAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.87),
                    ("west".to_string(), 1.04),
                    ("central".to_string(), 1.31),
                ]),
            }),
            Rc::new(TieredEscalationReservationAdjuster {
                bounds: vec![2116.0, 6096.0, 6415.0, 8250.0],
                rates: vec![0.97, 1.04, 1.05, 0.95],
            }),
            Rc::new(CapEscalationReservationAdjuster { cap: 26205.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 45724.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_reservation_engine();
        let item = new_escalation_reservation("code-508", "label-278");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
