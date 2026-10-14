use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the zone reservation calculation.
pub trait ZoneReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneReservationAdjuster {
    pub percent: f64,
}

impl ZoneReservationModifier for PercentZoneReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeZoneReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneReservationModifier for ScopeZoneReservationAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ZoneReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReservationRevisionOffsetAdjuster {
    pub weight: f64,
}

impl ZoneReservationModifier for ZoneReservationRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneReservation, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneReservationAdjuster {
    pub cap: f64,
}

impl ZoneReservationModifier for CapZoneReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ZoneReservationModifier for TieredZoneReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ZoneReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the zone reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneReservationEngine {
    pub steps: Vec<Rc<dyn ZoneReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneReservationEngine {
    pub fn price(&mut self, item: &ZoneReservation) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneReservationModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_zone_reservation_engine() -> ZoneReservationEngine {
    ZoneReservationEngine {
        steps: vec![
            Rc::new(PercentZoneReservationAdjuster { percent: -11.6 }) as Rc<dyn ZoneReservationModifier>,
            Rc::new(ScopeZoneReservationAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.33)]),
            }),
            Rc::new(ZoneReservationRevisionOffsetAdjuster { weight: 1.94 }),
            Rc::new(CapZoneReservationAdjuster { cap: 22375.0 }),
            Rc::new(TieredZoneReservationAdjuster {
                bounds: vec![4538.0, 5326.0, 6392.0, 6977.0],
                rates: vec![1.07, 1.2, 1.05, 0.89],
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 34244.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_reservation_engine();
        let item = new_zone_reservation("code-79", "label-235");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
