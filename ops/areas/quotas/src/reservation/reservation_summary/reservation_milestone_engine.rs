use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the reservation milestone calculation.
pub trait ReservationMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMilestoneQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl ReservationMilestoneStep for ReservationMilestoneQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationMilestone, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitReservationMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationMilestoneStep for UnitReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ReservationMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationMilestoneAdjuster {
    pub percent: f64,
}

impl ReservationMilestoneStep for PercentReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReservationMilestoneStep for TieredReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReservationMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationMilestoneAdjuster {
    pub step: f64,
}

impl ReservationMilestoneStep for RoundingReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the reservation milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationMilestoneEngine {
    pub steps: Vec<Rc<dyn ReservationMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationMilestoneEngine {
    pub fn compute(&mut self, item: &ReservationMilestone) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_reservation_milestone_engine() -> ReservationMilestoneEngine {
    ReservationMilestoneEngine {
        steps: vec![
            Rc::new(ReservationMilestoneQuotaUnitsOffsetAdjuster { weight: 1.44 }) as Rc<dyn ReservationMilestoneStep>,
            Rc::new(UnitReservationMilestoneAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.16), ("north".to_string(), 0.96)]),
            }),
            Rc::new(PercentReservationMilestoneAdjuster { percent: -12.8 }),
            Rc::new(TieredReservationMilestoneAdjuster {
                bounds: vec![1172.0, 4339.0, 6961.0, 7296.0],
                rates: vec![1.09, 1.09, 1.16, 1.19],
            }),
            Rc::new(RoundingReservationMilestoneAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 78004.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_milestone_engine();
        let item = new_reservation_milestone("code-939", "name-802");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
