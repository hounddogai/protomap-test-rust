use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the peering reservation calculation.
pub trait PeeringReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePeeringReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PeeringReservationFactor for NamePeeringReservationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PeeringReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringReservationFactor for TieredPeeringReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringReservationAdjuster {
    pub percent: f64,
}

impl PeeringReservationFactor for PercentPeeringReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringReservationEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl PeeringReservationFactor for PeeringReservationEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringReservation, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

/// Runs the peering reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringReservationEngine {
    pub steps: Vec<Rc<dyn PeeringReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringReservationEngine {
    pub fn evaluate(&mut self, item: &PeeringReservation) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PeeringReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_peering_reservation_engine() -> PeeringReservationEngine {
    PeeringReservationEngine {
        steps: vec![
            Rc::new(NamePeeringReservationAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.8),
                    ("central".to_string(), 1.24),
                    ("north".to_string(), 0.81),
                ]),
            }) as Rc<dyn PeeringReservationFactor>,
            Rc::new(TieredPeeringReservationAdjuster {
                bounds: vec![2986.0, 3701.0, 4703.0],
                rates: vec![1.19, 0.86, 1.12],
            }),
            Rc::new(PercentPeeringReservationAdjuster { percent: 6.0 }),
            Rc::new(PeeringReservationEndsAtOffsetAdjuster { weight: 0.84 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 82622.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_reservation_engine();
        let item = new_peering_reservation("code-362", "name-587");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
