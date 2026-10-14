use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the forecast profile calculation.
pub trait ForecastProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastProfileAdjuster {
    pub step: f64,
}

impl ForecastProfileStep for RoundingForecastProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastProfileAdjuster {
    pub cap: f64,
}

impl ForecastProfileStep for CapForecastProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedForecastProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ForecastProfileStep for FlaggedForecastProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ForecastProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastProfileAdjuster {
    pub minimum: f64,
}

impl ForecastProfileStep for MinimumForecastProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the forecast profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastProfileEngine {
    pub steps: Vec<Rc<dyn ForecastProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastProfileEngine {
    pub fn evaluate(&mut self, item: &ForecastProfile) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ForecastProfileStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ForecastProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_forecast_profile_engine(flags: &flags::Registry) -> ForecastProfileEngine {
    ForecastProfileEngine {
        steps: vec![
            Rc::new(RoundingForecastProfileAdjuster { step: 0.25 }) as Rc<dyn ForecastProfileStep>,
            Rc::new(CapForecastProfileAdjuster { cap: 32956.0 }),
            Rc::new(FlaggedForecastProfileAdjuster {
                flags: flags.clone(),
                flag: "forecast-profile-seasonal".to_string(),
                percent: 0.6,
            }),
            Rc::new(MinimumForecastProfileAdjuster { minimum: 27.6 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 80562.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_profile_engine(&flags::new_registry());
        let item = new_forecast_profile("code-904", "name-753");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
