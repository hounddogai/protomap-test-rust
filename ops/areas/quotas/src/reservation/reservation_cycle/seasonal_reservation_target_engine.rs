use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal reservation target calculation.
pub trait SeasonalReservationTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalReservationTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalReservationTargetAdjuster {
    pub step: f64,
}

impl SeasonalReservationTargetAdjuster for RoundingSeasonalReservationTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalReservationTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationTargetRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalReservationTargetAdjuster for SeasonalReservationTargetRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalReservationTarget, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalReservationTargetAdjuster {
    pub percent: f64,
}

impl SeasonalReservationTargetAdjuster for PercentSeasonalReservationTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalReservationTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalReservationTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalReservationTargetAdjuster for TieredSeasonalReservationTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalReservationTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalReservationTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalReservationTargetAdjuster for FlaggedSeasonalReservationTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalReservationTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSeasonalReservationTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalReservationTargetAdjuster for LabelSeasonalReservationTargetAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SeasonalReservationTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal reservation target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalReservationTargetEngine {
    pub steps: Vec<Rc<dyn SeasonalReservationTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalReservationTargetEngine {
    pub fn price(&mut self, item: &SeasonalReservationTarget) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalReservationTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_reservation_target_engine(flags: &flags::Registry) -> SeasonalReservationTargetEngine {
    SeasonalReservationTargetEngine {
        steps: vec![
            Rc::new(RoundingSeasonalReservationTargetAdjuster { step: 1.0 })
                as Rc<dyn SeasonalReservationTargetAdjuster>,
            Rc::new(SeasonalReservationTargetRotationDaysOffsetAdjuster { weight: 0.03 }),
            Rc::new(PercentSeasonalReservationTargetAdjuster { percent: -8.6 }),
            Rc::new(TieredSeasonalReservationTargetAdjuster {
                bounds: vec![55.0, 2319.0],
                rates: vec![0.85, 1.0],
            }),
            Rc::new(FlaggedSeasonalReservationTargetAdjuster {
                flags: flags.clone(),
                flag: "seasonal-reservation-target-batched".to_string(),
                percent: -7.4,
            }),
            Rc::new(LabelSeasonalReservationTargetAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.25),
                    ("south".to_string(), 1.37),
                    ("central".to_string(), 1.2),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 5961.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_reservation_target_engine(&flags::new_registry());
        let item = new_seasonal_reservation_target("code-314", "label-452");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
