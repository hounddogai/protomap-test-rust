use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional reservation cycle calculation.
pub trait RegionalReservationCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalReservationCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceRegionalReservationCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalReservationCycleFactor for NamespaceRegionalReservationCycleAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &RegionalReservationCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalReservationCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalReservationCycleFactor for TieredRegionalReservationCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalReservationCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalReservationCycleAdjuster {
    pub minimum: f64,
}

impl RegionalReservationCycleFactor for MinimumRegionalReservationCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalReservationCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalReservationCycleAdjuster {
    pub percent: f64,
}

impl RegionalReservationCycleFactor for PercentRegionalReservationCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalReservationCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalReservationCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalReservationCycleFactor for FlaggedRegionalReservationCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalReservationCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional reservation cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalReservationCycleEngine {
    pub steps: Vec<Rc<dyn RegionalReservationCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalReservationCycleEngine {
    pub fn compute(&mut self, item: &RegionalReservationCycle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RegionalReservationCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_reservation_cycle_engine(flags: &flags::Registry) -> RegionalReservationCycleEngine {
    RegionalReservationCycleEngine {
        steps: vec![
            Rc::new(NamespaceRegionalReservationCycleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.36), ("east".to_string(), 1.13)]),
            }) as Rc<dyn RegionalReservationCycleFactor>,
            Rc::new(TieredRegionalReservationCycleAdjuster {
                bounds: vec![3639.0, 5565.0],
                rates: vec![1.09, 0.86],
            }),
            Rc::new(MinimumRegionalReservationCycleAdjuster { minimum: 17.4 }),
            Rc::new(PercentRegionalReservationCycleAdjuster { percent: 3.8 }),
            Rc::new(FlaggedRegionalReservationCycleAdjuster {
                flags: flags.clone(),
                flag: "regional-reservation-cycle-adaptive".to_string(),
                percent: 5.8,
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 23846.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_reservation_cycle_engine(&flags::new_registry());
        let item = new_regional_reservation_cycle("code-406", "label-916");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
