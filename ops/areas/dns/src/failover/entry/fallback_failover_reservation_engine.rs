use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback failover reservation calculation.
pub trait FallbackFailoverReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackFailoverReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackFailoverReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackFailoverReservationStep for TieredFallbackFailoverReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackFailoverReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackFailoverReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackFailoverReservationStep for FlaggedFallbackFailoverReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackFailoverReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitFallbackFailoverReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackFailoverReservationStep for UnitFallbackFailoverReservationAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &FallbackFailoverReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback failover reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackFailoverReservationEngine {
    pub steps: Vec<Rc<dyn FallbackFailoverReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackFailoverReservationEngine {
    pub fn compute(&mut self, item: &FallbackFailoverReservation) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackFailoverReservationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackFailoverReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_failover_reservation_engine(flags: &flags::Registry) -> FallbackFailoverReservationEngine {
    FallbackFailoverReservationEngine {
        steps: vec![
            Rc::new(TieredFallbackFailoverReservationAdjuster {
                bounds: vec![5520.0, 7214.0, 8851.0],
                rates: vec![0.81, 1.11, 1.19],
            }) as Rc<dyn FallbackFailoverReservationStep>,
            Rc::new(FlaggedFallbackFailoverReservationAdjuster {
                flags: flags.clone(),
                flag: "fallback-failover-reservation-global".to_string(),
                percent: -6.7,
            }),
            Rc::new(UnitFallbackFailoverReservationAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.21),
                    ("retail".to_string(), 0.77),
                    ("south".to_string(), 1.39),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 13826.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_failover_reservation_engine(&flags::new_registry());
        let item = new_fallback_failover_reservation("code-987", "name-111");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
