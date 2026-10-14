use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval reservation calculation.
pub trait ApprovalReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalReservationModifier for TieredApprovalReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalReservationTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalReservationModifier for ApprovalReservationTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalReservation, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleApprovalReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalReservationModifier for TitleApprovalReservationAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ApprovalReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the approval reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalReservationEngine {
    pub steps: Vec<Rc<dyn ApprovalReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalReservationEngine {
    pub fn run(&mut self, item: &ApprovalReservation) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_approval_reservation_engine() -> ApprovalReservationEngine {
    ApprovalReservationEngine {
        steps: vec![
            Rc::new(TieredApprovalReservationAdjuster {
                bounds: vec![6854.0, 8801.0],
                rates: vec![0.97, 1.0],
            }) as Rc<dyn ApprovalReservationModifier>,
            Rc::new(ApprovalReservationTtlSecondsOffsetAdjuster { weight: 1.94 }),
            Rc::new(TitleApprovalReservationAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.81),
                    ("central".to_string(), 1.03),
                    ("north".to_string(), 0.73),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 51465.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_reservation_engine();
        let item = new_approval_reservation("code-294", "title-577");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
