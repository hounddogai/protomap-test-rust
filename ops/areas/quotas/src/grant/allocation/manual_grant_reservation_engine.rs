use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual grant reservation calculation.
pub trait ManualGrantReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualGrantReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantReservationCeilingOffsetAdjuster {
    pub weight: f64,
}

impl ManualGrantReservationAdjuster for ManualGrantReservationCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &ManualGrantReservation, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualGrantReservationAdjuster {
    pub step: f64,
}

impl ManualGrantReservationAdjuster for RoundingManualGrantReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualGrantReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelManualGrantReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualGrantReservationAdjuster for ChannelManualGrantReservationAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &ManualGrantReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualGrantReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualGrantReservationAdjuster for TieredManualGrantReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualGrantReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualGrantReservationAdjuster {
    pub cap: f64,
}

impl ManualGrantReservationAdjuster for CapManualGrantReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualGrantReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualGrantReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualGrantReservationAdjuster for FlaggedManualGrantReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualGrantReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual grant reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualGrantReservationEngine {
    pub steps: Vec<Rc<dyn ManualGrantReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualGrantReservationEngine {
    pub fn run(&mut self, item: &ManualGrantReservation) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualGrantReservationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualGrantReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_grant_reservation_engine(flags: &flags::Registry) -> ManualGrantReservationEngine {
    ManualGrantReservationEngine {
        steps: vec![
            Rc::new(ManualGrantReservationCeilingOffsetAdjuster { weight: 1.84 })
                as Rc<dyn ManualGrantReservationAdjuster>,
            Rc::new(RoundingManualGrantReservationAdjuster { step: 0.5 }),
            Rc::new(ChannelManualGrantReservationAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.38), ("west".to_string(), 0.92)]),
            }),
            Rc::new(TieredManualGrantReservationAdjuster {
                bounds: vec![657.0, 2205.0, 2783.0, 4132.0],
                rates: vec![1.14, 0.99, 0.97, 0.9],
            }),
            Rc::new(CapManualGrantReservationAdjuster { cap: 20248.0 }),
            Rc::new(FlaggedManualGrantReservationAdjuster {
                flags: flags.clone(),
                flag: "manual-grant-reservation-legacy".to_string(),
                percent: 5.2,
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 83409.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_grant_reservation_engine(&flags::new_registry());
        let item = new_manual_grant_reservation("code-913", "label-622");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
