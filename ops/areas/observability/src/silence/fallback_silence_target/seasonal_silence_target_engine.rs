use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal silence target calculation.
pub trait SeasonalSilenceTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalSilenceTarget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalSilenceTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalSilenceTargetAdjuster for FlaggedSeasonalSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalSilenceTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalSilenceTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalSilenceTargetAdjuster for TieredSeasonalSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalSilenceTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalSilenceTargetAdjuster {
    pub step: f64,
}

impl SeasonalSilenceTargetAdjuster for RoundingSeasonalSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalSilenceTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the seasonal silence target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalSilenceTargetEngine {
    pub steps: Vec<Rc<dyn SeasonalSilenceTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalSilenceTargetEngine {
    pub fn evaluate(&mut self, item: &SeasonalSilenceTarget) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalSilenceTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalSilenceTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_silence_target_engine(flags: &flags::Registry) -> SeasonalSilenceTargetEngine {
    SeasonalSilenceTargetEngine {
        steps: vec![
            Rc::new(FlaggedSeasonalSilenceTargetAdjuster {
                flags: flags.clone(),
                flag: "seasonal-silence-target-primary".to_string(),
                percent: 3.7,
            }) as Rc<dyn SeasonalSilenceTargetAdjuster>,
            Rc::new(TieredSeasonalSilenceTargetAdjuster {
                bounds: vec![315.0, 9560.0],
                rates: vec![0.93, 1.05],
            }),
            Rc::new(RoundingSeasonalSilenceTargetAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 35850.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_silence_target_engine(&flags::new_registry());
        let item = new_seasonal_silence_target("code-645", "name-102");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
