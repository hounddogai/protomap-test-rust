use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default forecast notice calculation.
pub trait DefaultForecastNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultForecastNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultForecastNoticeAdjuster {
    pub percent: f64,
}

impl DefaultForecastNoticeStep for PercentDefaultForecastNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultForecastNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastNoticeBudgetOffsetAdjuster {
    pub weight: f64,
}

impl DefaultForecastNoticeStep for DefaultForecastNoticeBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultForecastNotice, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDefaultForecastNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultForecastNoticeStep for LabelDefaultForecastNoticeAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DefaultForecastNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultForecastNoticeAdjuster {
    pub minimum: f64,
}

impl DefaultForecastNoticeStep for MinimumDefaultForecastNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultForecastNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultForecastNoticeAdjuster {
    pub step: f64,
}

impl DefaultForecastNoticeStep for RoundingDefaultForecastNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultForecastNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultForecastNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultForecastNoticeStep for FlaggedDefaultForecastNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultForecastNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default forecast notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultForecastNoticeEngine {
    pub steps: Vec<Rc<dyn DefaultForecastNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultForecastNoticeEngine {
    pub fn price(&mut self, item: &DefaultForecastNotice) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultForecastNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_forecast_notice_engine(flags: &flags::Registry) -> DefaultForecastNoticeEngine {
    DefaultForecastNoticeEngine {
        steps: vec![
            Rc::new(PercentDefaultForecastNoticeAdjuster { percent: -7.6 }) as Rc<dyn DefaultForecastNoticeStep>,
            Rc::new(DefaultForecastNoticeBudgetOffsetAdjuster { weight: 1.24 }),
            Rc::new(LabelDefaultForecastNoticeAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.87),
                    ("online".to_string(), 1.36),
                    ("central".to_string(), 0.72),
                ]),
            }),
            Rc::new(MinimumDefaultForecastNoticeAdjuster { minimum: 15.0 }),
            Rc::new(RoundingDefaultForecastNoticeAdjuster { step: 0.05 }),
            Rc::new(FlaggedDefaultForecastNoticeAdjuster {
                flags: flags.clone(),
                flag: "default-forecast-notice-fallback".to_string(),
                percent: 9.7,
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 24071.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_forecast_notice_engine(&flags::new_registry());
        let item = new_default_forecast_notice("code-788", "label-29");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
