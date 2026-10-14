use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist reservation calculation.
pub trait AllowlistReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAllowlistReservationAdjuster {
    pub percent: f64,
}

impl AllowlistReservationStep for PercentAllowlistReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AllowlistReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAllowlistReservationAdjuster {
    pub minimum: f64,
}

impl AllowlistReservationStep for MinimumAllowlistReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AllowlistReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterAllowlistReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AllowlistReservationStep for ClusterAllowlistReservationAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &AllowlistReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistReservationStep for FlaggedAllowlistReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAllowlistReservationAdjuster {
    pub step: f64,
}

impl AllowlistReservationStep for RoundingAllowlistReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AllowlistReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistReservationStep for TieredAllowlistReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the allowlist reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistReservationEngine {
    pub steps: Vec<Rc<dyn AllowlistReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistReservationEngine {
    pub fn run(&mut self, item: &AllowlistReservation) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AllowlistReservationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_allowlist_reservation_engine(flags: &flags::Registry) -> AllowlistReservationEngine {
    AllowlistReservationEngine {
        steps: vec![
            Rc::new(PercentAllowlistReservationAdjuster { percent: 21.4 }) as Rc<dyn AllowlistReservationStep>,
            Rc::new(MinimumAllowlistReservationAdjuster { minimum: 14.5 }),
            Rc::new(ClusterAllowlistReservationAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.85), ("west".to_string(), 1.18)]),
            }),
            Rc::new(FlaggedAllowlistReservationAdjuster {
                flags: flags.clone(),
                flag: "allowlist-reservation-pending".to_string(),
                percent: 2.9,
            }),
            Rc::new(RoundingAllowlistReservationAdjuster { step: 0.5 }),
            Rc::new(TieredAllowlistReservationAdjuster {
                bounds: vec![553.0, 7570.0],
                rates: vec![1.06, 1.11],
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 72657.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_reservation_engine(&flags::new_registry());
        let item = new_allowlist_reservation("code-641", "label-838");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
