use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional reservation claim calculation.
pub trait RegionalReservationClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalReservationClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalReservationClaimAdjuster {
    pub cap: f64,
}

impl RegionalReservationClaimFactor for CapRegionalReservationClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalReservationClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalReservationClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalReservationClaimFactor for FlaggedRegionalReservationClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalReservationClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalReservationClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalReservationClaimFactor for TieredRegionalReservationClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalReservationClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalReservationClaimAdjuster {
    pub minimum: f64,
}

impl RegionalReservationClaimFactor for MinimumRegionalReservationClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalReservationClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationClaimAmountOffsetAdjuster {
    pub weight: f64,
}

impl RegionalReservationClaimFactor for RegionalReservationClaimAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalReservationClaim, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the regional reservation claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalReservationClaimEngine {
    pub steps: Vec<Rc<dyn RegionalReservationClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalReservationClaimEngine {
    pub fn compute(&mut self, item: &RegionalReservationClaim) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalReservationClaimFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalReservationClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_reservation_claim_engine(flags: &flags::Registry) -> RegionalReservationClaimEngine {
    RegionalReservationClaimEngine {
        steps: vec![
            Rc::new(CapRegionalReservationClaimAdjuster { cap: 33592.0 }) as Rc<dyn RegionalReservationClaimFactor>,
            Rc::new(FlaggedRegionalReservationClaimAdjuster {
                flags: flags.clone(),
                flag: "regional-reservation-claim-tiered".to_string(),
                percent: 7.1,
            }),
            Rc::new(TieredRegionalReservationClaimAdjuster {
                bounds: vec![3478.0, 4763.0, 7731.0, 9273.0],
                rates: vec![1.18, 1.13, 0.99, 1.05],
            }),
            Rc::new(MinimumRegionalReservationClaimAdjuster { minimum: 29.8 }),
            Rc::new(RegionalReservationClaimAmountOffsetAdjuster { weight: 1.21 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 17564.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_reservation_claim_engine(&flags::new_registry());
        let item = new_regional_reservation_claim("code-173", "title-244");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
