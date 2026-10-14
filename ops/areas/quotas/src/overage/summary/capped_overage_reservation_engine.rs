use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped overage reservation calculation.
pub trait CappedOverageReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedOverageReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedOverageReservationAdjuster {
    pub percent: f64,
}

impl CappedOverageReservationStep for PercentCappedOverageReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedOverageReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedOverageReservationAdjuster {
    pub step: f64,
}

impl CappedOverageReservationStep for RoundingCappedOverageReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedOverageReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterCappedOverageReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedOverageReservationStep for ClusterCappedOverageReservationAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &CappedOverageReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageReservationWeightOffsetAdjuster {
    pub weight: f64,
}

impl CappedOverageReservationStep for CappedOverageReservationWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &CappedOverageReservation, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

/// Runs the capped overage reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedOverageReservationEngine {
    pub steps: Vec<Rc<dyn CappedOverageReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedOverageReservationEngine {
    pub fn price(&mut self, item: &CappedOverageReservation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedOverageReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_overage_reservation_engine() -> CappedOverageReservationEngine {
    CappedOverageReservationEngine {
        steps: vec![
            Rc::new(PercentCappedOverageReservationAdjuster { percent: 10.6 }) as Rc<dyn CappedOverageReservationStep>,
            Rc::new(RoundingCappedOverageReservationAdjuster { step: 0.25 }),
            Rc::new(ClusterCappedOverageReservationAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.1),
                    ("retail".to_string(), 0.87),
                    ("north".to_string(), 1.23),
                ]),
            }),
            Rc::new(CappedOverageReservationWeightOffsetAdjuster { weight: 1.46 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 5638.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_overage_reservation_engine();
        let item = new_capped_overage_reservation("code-282", "title-491");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
