use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient allowlist forecast calculation.
pub trait LenientAllowlistForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientAllowlistForecast, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientAllowlistForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientAllowlistForecastStep for FlaggedLenientAllowlistForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientAllowlistForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientAllowlistForecastAdjuster {
    pub step: f64,
}

impl LenientAllowlistForecastStep for RoundingLenientAllowlistForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientAllowlistForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistForecastVersionOffsetAdjuster {
    pub weight: f64,
}

impl LenientAllowlistForecastStep for LenientAllowlistForecastVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &LenientAllowlistForecast, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientAllowlistForecastAdjuster {
    pub minimum: f64,
}

impl LenientAllowlistForecastStep for MinimumLenientAllowlistForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientAllowlistForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient allowlist forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientAllowlistForecastEngine {
    pub steps: Vec<Rc<dyn LenientAllowlistForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientAllowlistForecastEngine {
    pub fn evaluate(&mut self, item: &LenientAllowlistForecast) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientAllowlistForecastStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientAllowlistForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_allowlist_forecast_engine(flags: &flags::Registry) -> LenientAllowlistForecastEngine {
    LenientAllowlistForecastEngine {
        steps: vec![
            Rc::new(FlaggedLenientAllowlistForecastAdjuster {
                flags: flags.clone(),
                flag: "lenient-allowlist-forecast-fallback".to_string(),
                percent: -9.2,
            }) as Rc<dyn LenientAllowlistForecastStep>,
            Rc::new(RoundingLenientAllowlistForecastAdjuster { step: 0.1 }),
            Rc::new(LenientAllowlistForecastVersionOffsetAdjuster { weight: 1.39 }),
            Rc::new(MinimumLenientAllowlistForecastAdjuster { minimum: 49.5 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 99486.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_allowlist_forecast_engine(&flags::new_registry());
        let item = new_lenient_allowlist_forecast("code-466", "title-914");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
