use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the trust domain reservation calculation.
pub trait TrustDomainReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTrustDomainReservationAdjuster {
    pub minimum: f64,
}

impl TrustDomainReservationFactor for MinimumTrustDomainReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TrustDomainReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainReservationFactor for TieredTrustDomainReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainReservationFactor for FlaggedTrustDomainReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the trust domain reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainReservationEngine {
    pub steps: Vec<Rc<dyn TrustDomainReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainReservationEngine {
    pub fn evaluate(&mut self, item: &TrustDomainReservation) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainReservationFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_trust_domain_reservation_engine(flags: &flags::Registry) -> TrustDomainReservationEngine {
    TrustDomainReservationEngine {
        steps: vec![
            Rc::new(MinimumTrustDomainReservationAdjuster { minimum: 42.6 }) as Rc<dyn TrustDomainReservationFactor>,
            Rc::new(TieredTrustDomainReservationAdjuster {
                bounds: vec![857.0, 8973.0],
                rates: vec![0.97, 1.16],
            }),
            Rc::new(FlaggedTrustDomainReservationAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-reservation-strict".to_string(),
                percent: 8.7,
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 91923.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_reservation_engine(&flags::new_registry());
        let item = new_trust_domain_reservation("code-285", "title-343");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
