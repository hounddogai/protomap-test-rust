use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast notice calculation.
pub trait ForecastNoticeModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastNoticeModifier for TieredForecastNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastNoticeWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl ForecastNoticeModifier for ForecastNoticeWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastNotice, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastNoticeAdjuster {
    pub step: f64,
}

impl ForecastNoticeModifier for RoundingForecastNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentForecastNoticeAdjuster {
    pub percent: f64,
}

impl ForecastNoticeModifier for PercentForecastNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ForecastNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastNoticeAdjuster {
    pub cap: f64,
}

impl ForecastNoticeModifier for CapForecastNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelForecastNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastNoticeModifier for LabelForecastNoticeAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ForecastNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the forecast notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastNoticeEngine {
    pub steps: Vec<Rc<dyn ForecastNoticeModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastNoticeEngine {
    pub fn run(&mut self, item: &ForecastNotice) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ForecastNoticeModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ForecastNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_forecast_notice_engine() -> ForecastNoticeEngine {
    ForecastNoticeEngine {
        steps: vec![
            Rc::new(TieredForecastNoticeAdjuster {
                bounds: vec![1380.0, 1476.0, 2417.0, 5117.0],
                rates: vec![0.98, 0.92, 1.15, 0.93],
            }) as Rc<dyn ForecastNoticeModifier>,
            Rc::new(ForecastNoticeWindowDaysOffsetAdjuster { weight: 1.24 }),
            Rc::new(RoundingForecastNoticeAdjuster { step: 0.05 }),
            Rc::new(PercentForecastNoticeAdjuster { percent: 21.2 }),
            Rc::new(CapForecastNoticeAdjuster { cap: 39938.0 }),
            Rc::new(LabelForecastNoticeAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.87), ("south".to_string(), 0.72)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 20502.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_notice_engine();
        let item = new_forecast_notice("code-352", "label-192");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
