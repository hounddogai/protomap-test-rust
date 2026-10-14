use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the reservation mapping calculation.
pub trait ReservationMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationMapping, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationMappingModifier for FlaggedReservationMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationMappingAdjuster {
    pub cap: f64,
}

impl ReservationMappingModifier for CapReservationMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationMappingAdjuster {
    pub percent: f64,
}

impl ReservationMappingModifier for PercentReservationMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReservationMappingModifier for TieredReservationMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReservationMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMappingCostOffsetAdjuster {
    pub weight: f64,
}

impl ReservationMappingModifier for ReservationMappingCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationMapping, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameReservationMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationMappingModifier for NameReservationMappingAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ReservationMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the reservation mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationMappingEngine {
    pub steps: Vec<Rc<dyn ReservationMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationMappingEngine {
    pub fn compute(&mut self, item: &ReservationMapping) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ReservationMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_reservation_mapping_engine(flags: &flags::Registry) -> ReservationMappingEngine {
    ReservationMappingEngine {
        steps: vec![
            Rc::new(FlaggedReservationMappingAdjuster {
                flags: flags.clone(),
                flag: "reservation-mapping-legacy".to_string(),
                percent: 9.3,
            }) as Rc<dyn ReservationMappingModifier>,
            Rc::new(CapReservationMappingAdjuster { cap: 21484.0 }),
            Rc::new(PercentReservationMappingAdjuster { percent: 16.9 }),
            Rc::new(TieredReservationMappingAdjuster {
                bounds: vec![715.0, 6418.0, 8108.0, 9195.0],
                rates: vec![1.06, 0.9, 1.07, 1.08],
            }),
            Rc::new(ReservationMappingCostOffsetAdjuster { weight: 1.63 }),
            Rc::new(NameReservationMappingAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.06), ("retail".to_string(), 1.26)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 81312.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_mapping_engine(&flags::new_registry());
        let item = new_reservation_mapping("code-444", "name-307");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
