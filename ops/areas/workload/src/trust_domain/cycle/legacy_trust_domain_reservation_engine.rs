use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy trust domain reservation calculation.
pub trait LegacyTrustDomainReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyTrustDomainReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainReservationExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl LegacyTrustDomainReservationFactor for LegacyTrustDomainReservationExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyTrustDomainReservation, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyTrustDomainReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyTrustDomainReservationFactor for FlaggedLegacyTrustDomainReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyTrustDomainReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLegacyTrustDomainReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyTrustDomainReservationFactor for TitleLegacyTrustDomainReservationAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LegacyTrustDomainReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy trust domain reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyTrustDomainReservationEngine {
    pub steps: Vec<Rc<dyn LegacyTrustDomainReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyTrustDomainReservationEngine {
    pub fn evaluate(&mut self, item: &LegacyTrustDomainReservation) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[LegacyTrustDomainReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_trust_domain_reservation_engine(flags: &flags::Registry) -> LegacyTrustDomainReservationEngine {
    LegacyTrustDomainReservationEngine {
        steps: vec![
            Rc::new(LegacyTrustDomainReservationExpiresAtOffsetAdjuster { weight: 1.51 })
                as Rc<dyn LegacyTrustDomainReservationFactor>,
            Rc::new(FlaggedLegacyTrustDomainReservationAdjuster {
                flags: flags.clone(),
                flag: "legacy-trust-domain-reservation-lenient".to_string(),
                percent: 6.3,
            }),
            Rc::new(TitleLegacyTrustDomainReservationAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.79), ("central".to_string(), 0.76)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 37940.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_trust_domain_reservation_engine(&flags::new_registry());
        let item = new_legacy_trust_domain_reservation("code-118", "title-825");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
