use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rolling verifier forecast calculation.
pub trait RollingVerifierForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingVerifierForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierForecastGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl RollingVerifierForecastAdjuster for RollingVerifierForecastGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &RollingVerifierForecast, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingVerifierForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingVerifierForecastAdjuster for FlaggedRollingVerifierForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingVerifierForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingVerifierForecastAdjuster {
    pub percent: f64,
}

impl RollingVerifierForecastAdjuster for PercentRollingVerifierForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingVerifierForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingVerifierForecastAdjuster {
    pub cap: f64,
}

impl RollingVerifierForecastAdjuster for CapRollingVerifierForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingVerifierForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling verifier forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingVerifierForecastEngine {
    pub steps: Vec<Rc<dyn RollingVerifierForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingVerifierForecastEngine {
    pub fn compute(&mut self, item: &RollingVerifierForecast) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingVerifierForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingVerifierForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_verifier_forecast_engine(flags: &flags::Registry) -> RollingVerifierForecastEngine {
    RollingVerifierForecastEngine {
        steps: vec![
            Rc::new(RollingVerifierForecastGraceDaysOffsetAdjuster { weight: 0.36 })
                as Rc<dyn RollingVerifierForecastAdjuster>,
            Rc::new(FlaggedRollingVerifierForecastAdjuster {
                flags: flags.clone(),
                flag: "rolling-verifier-forecast-weighted".to_string(),
                percent: 0.6,
            }),
            Rc::new(PercentRollingVerifierForecastAdjuster { percent: -23.5 }),
            Rc::new(CapRollingVerifierForecastAdjuster { cap: 17576.0 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 71132.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_verifier_forecast_engine(&flags::new_registry());
        let item = new_rolling_verifier_forecast("code-661", "label-936");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
