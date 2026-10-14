use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal freeze reservation calculation.
pub trait SeasonalFreezeReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalFreezeReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalFreezeReservationAdjuster {
    pub step: f64,
}

impl SeasonalFreezeReservationAdjuster for RoundingSeasonalFreezeReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalFreezeReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalFreezeReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalFreezeReservationAdjuster for FlaggedSeasonalFreezeReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalFreezeReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelSeasonalFreezeReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalFreezeReservationAdjuster for ChannelSeasonalFreezeReservationAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &SeasonalFreezeReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalFreezeReservationAdjuster {
    pub minimum: f64,
}

impl SeasonalFreezeReservationAdjuster for MinimumSeasonalFreezeReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalFreezeReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalFreezeReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalFreezeReservationAdjuster for TieredSeasonalFreezeReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalFreezeReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalFreezeReservationAdjuster {
    pub cap: f64,
}

impl SeasonalFreezeReservationAdjuster for CapSeasonalFreezeReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalFreezeReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the seasonal freeze reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalFreezeReservationEngine {
    pub steps: Vec<Rc<dyn SeasonalFreezeReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalFreezeReservationEngine {
    pub fn price(&mut self, item: &SeasonalFreezeReservation) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalFreezeReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_freeze_reservation_engine(flags: &flags::Registry) -> SeasonalFreezeReservationEngine {
    SeasonalFreezeReservationEngine {
        steps: vec![
            Rc::new(RoundingSeasonalFreezeReservationAdjuster { step: 0.25 })
                as Rc<dyn SeasonalFreezeReservationAdjuster>,
            Rc::new(FlaggedSeasonalFreezeReservationAdjuster {
                flags: flags.clone(),
                flag: "seasonal-freeze-reservation-batched".to_string(),
                percent: 9.6,
            }),
            Rc::new(ChannelSeasonalFreezeReservationAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.38),
                    ("east".to_string(), 0.93),
                    ("retail".to_string(), 1.39),
                ]),
            }),
            Rc::new(MinimumSeasonalFreezeReservationAdjuster { minimum: 5.9 }),
            Rc::new(TieredSeasonalFreezeReservationAdjuster {
                bounds: vec![666.0, 3508.0, 5199.0, 9809.0],
                rates: vec![1.16, 0.91, 1.13, 1.11],
            }),
            Rc::new(CapSeasonalFreezeReservationAdjuster { cap: 4200.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 5725.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_freeze_reservation_engine(&flags::new_registry());
        let item = new_seasonal_freeze_reservation("code-138", "label-55");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
