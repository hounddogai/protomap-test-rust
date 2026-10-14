use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal freeze entry calculation.
pub trait SeasonalFreezeEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalFreezeEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalFreezeEntryAdjuster {
    pub percent: f64,
}

impl SeasonalFreezeEntryStep for PercentSeasonalFreezeEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalFreezeEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalFreezeEntryAdjuster {
    pub minimum: f64,
}

impl SeasonalFreezeEntryStep for MinimumSeasonalFreezeEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalFreezeEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalFreezeEntryAdjuster {
    pub step: f64,
}

impl SeasonalFreezeEntryStep for RoundingSeasonalFreezeEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalFreezeEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalFreezeEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalFreezeEntryStep for FlaggedSeasonalFreezeEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalFreezeEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalFreezeEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalFreezeEntryStep for TieredSeasonalFreezeEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalFreezeEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeEntryGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalFreezeEntryStep for SeasonalFreezeEntryGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalFreezeEntry, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the seasonal freeze entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalFreezeEntryEngine {
    pub steps: Vec<Rc<dyn SeasonalFreezeEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalFreezeEntryEngine {
    pub fn price(&mut self, item: &SeasonalFreezeEntry) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalFreezeEntryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalFreezeEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_freeze_entry_engine(flags: &flags::Registry) -> SeasonalFreezeEntryEngine {
    SeasonalFreezeEntryEngine {
        steps: vec![
            Rc::new(PercentSeasonalFreezeEntryAdjuster { percent: -5.2 }) as Rc<dyn SeasonalFreezeEntryStep>,
            Rc::new(MinimumSeasonalFreezeEntryAdjuster { minimum: 37.9 }),
            Rc::new(RoundingSeasonalFreezeEntryAdjuster { step: 0.05 }),
            Rc::new(FlaggedSeasonalFreezeEntryAdjuster {
                flags: flags.clone(),
                flag: "seasonal-freeze-entry-rolling".to_string(),
                percent: -7.9,
            }),
            Rc::new(TieredSeasonalFreezeEntryAdjuster {
                bounds: vec![442.0, 3409.0, 4567.0, 7162.0],
                rates: vec![0.82, 0.82, 0.91, 0.98],
            }),
            Rc::new(SeasonalFreezeEntryGraceDaysOffsetAdjuster { weight: 0.11 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 95955.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_freeze_entry_engine(&flags::new_registry());
        let item = new_seasonal_freeze_entry("code-161", "name-876");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
