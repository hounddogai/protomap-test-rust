use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy federation reservation calculation.
pub trait LegacyFederationReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyFederationReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyFederationReservationAdjuster {
    pub cap: f64,
}

impl LegacyFederationReservationStep for CapLegacyFederationReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyFederationReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyFederationReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyFederationReservationStep for FlaggedLegacyFederationReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyFederationReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyFederationReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyFederationReservationStep for TieredLegacyFederationReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyFederationReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLegacyFederationReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyFederationReservationStep for NameLegacyFederationReservationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LegacyFederationReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy federation reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyFederationReservationEngine {
    pub steps: Vec<Rc<dyn LegacyFederationReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyFederationReservationEngine {
    pub fn run(&mut self, item: &LegacyFederationReservation) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[LegacyFederationReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_federation_reservation_engine(flags: &flags::Registry) -> LegacyFederationReservationEngine {
    LegacyFederationReservationEngine {
        steps: vec![
            Rc::new(CapLegacyFederationReservationAdjuster { cap: 9570.0 }) as Rc<dyn LegacyFederationReservationStep>,
            Rc::new(FlaggedLegacyFederationReservationAdjuster {
                flags: flags.clone(),
                flag: "legacy-federation-reservation-strict".to_string(),
                percent: -5.6,
            }),
            Rc::new(TieredLegacyFederationReservationAdjuster {
                bounds: vec![2769.0, 3239.0, 8625.0, 9003.0],
                rates: vec![0.96, 1.13, 1.01, 0.83],
            }),
            Rc::new(NameLegacyFederationReservationAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.31),
                    ("east".to_string(), 1.25),
                    ("online".to_string(), 1.01),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 80672.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_federation_reservation_engine(&flags::new_registry());
        let item = new_legacy_federation_reservation("code-60", "name-748");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
