use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary ttl policy reservation calculation.
pub trait PrimaryTtlPolicyReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryTtlPolicyReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryTtlPolicyReservationAdjuster {
    pub minimum: f64,
}

impl PrimaryTtlPolicyReservationFactor for MinimumPrimaryTtlPolicyReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryTtlPolicyReservationAdjuster {
    pub step: f64,
}

impl PrimaryTtlPolicyReservationFactor for RoundingPrimaryTtlPolicyReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryPrimaryTtlPolicyReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryTtlPolicyReservationFactor for CountryPrimaryTtlPolicyReservationAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &PrimaryTtlPolicyReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyReservationBudgetOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryTtlPolicyReservationFactor for PrimaryTtlPolicyReservationBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyReservation, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryTtlPolicyReservationAdjuster {
    pub percent: f64,
}

impl PrimaryTtlPolicyReservationFactor for PercentPrimaryTtlPolicyReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryTtlPolicyReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryTtlPolicyReservationFactor for FlaggedPrimaryTtlPolicyReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary ttl policy reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryTtlPolicyReservationEngine {
    pub steps: Vec<Rc<dyn PrimaryTtlPolicyReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryTtlPolicyReservationEngine {
    pub fn compute(&mut self, item: &PrimaryTtlPolicyReservation) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryTtlPolicyReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_ttl_policy_reservation_engine(flags: &flags::Registry) -> PrimaryTtlPolicyReservationEngine {
    PrimaryTtlPolicyReservationEngine {
        steps: vec![
            Rc::new(MinimumPrimaryTtlPolicyReservationAdjuster { minimum: 42.6 })
                as Rc<dyn PrimaryTtlPolicyReservationFactor>,
            Rc::new(RoundingPrimaryTtlPolicyReservationAdjuster { step: 0.25 }),
            Rc::new(CountryPrimaryTtlPolicyReservationAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.29), ("online".to_string(), 0.78)]),
            }),
            Rc::new(PrimaryTtlPolicyReservationBudgetOffsetAdjuster { weight: 0.79 }),
            Rc::new(PercentPrimaryTtlPolicyReservationAdjuster { percent: 15.5 }),
            Rc::new(FlaggedPrimaryTtlPolicyReservationAdjuster {
                flags: flags.clone(),
                flag: "primary-ttl-policy-reservation-regional".to_string(),
                percent: 7.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 73434.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_ttl_policy_reservation_engine(&flags::new_registry());
        let item = new_primary_ttl_policy_reservation("code-177", "title-730");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
