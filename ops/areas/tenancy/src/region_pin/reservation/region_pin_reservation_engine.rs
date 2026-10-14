use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the region pin reservation calculation.
pub trait RegionPinReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinReservationAdjuster {
    pub cap: f64,
}

impl RegionPinReservationStep for CapRegionPinReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinReservationAdjuster {
    pub minimum: f64,
}

impl RegionPinReservationStep for MinimumRegionPinReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRegionPinReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinReservationStep for LabelRegionPinReservationAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RegionPinReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinReservationAdjuster {
    pub step: f64,
}

impl RegionPinReservationStep for RoundingRegionPinReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReservationAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinReservationStep for RegionPinReservationAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinReservation, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

/// Runs the region pin reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinReservationEngine {
    pub steps: Vec<Rc<dyn RegionPinReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinReservationEngine {
    pub fn evaluate(&mut self, item: &RegionPinReservation) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionPinReservationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_region_pin_reservation_engine() -> RegionPinReservationEngine {
    RegionPinReservationEngine {
        steps: vec![
            Rc::new(CapRegionPinReservationAdjuster { cap: 4113.0 }) as Rc<dyn RegionPinReservationStep>,
            Rc::new(MinimumRegionPinReservationAdjuster { minimum: 33.8 }),
            Rc::new(LabelRegionPinReservationAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.3)]),
            }),
            Rc::new(RoundingRegionPinReservationAdjuster { step: 0.05 }),
            Rc::new(RegionPinReservationAttemptsOffsetAdjuster { weight: 1.83 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 16255.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_reservation_engine();
        let item = new_region_pin_reservation("code-491", "label-108");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
