use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending page forecast calculation.
pub trait PendingPageForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPageForecast, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingPageForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingPageForecastModifier for FlaggedPendingPageForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingPageForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingPageForecastAdjuster {
    pub step: f64,
}

impl PendingPageForecastModifier for RoundingPendingPageForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingPageForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageForecastMaximumOffsetAdjuster {
    pub weight: f64,
}

impl PendingPageForecastModifier for PendingPageForecastMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &PendingPageForecast, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

/// Runs the pending page forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPageForecastEngine {
    pub steps: Vec<Rc<dyn PendingPageForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPageForecastEngine {
    pub fn run(&mut self, item: &PendingPageForecast) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingPageForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingPageForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_page_forecast_engine(flags: &flags::Registry) -> PendingPageForecastEngine {
    PendingPageForecastEngine {
        steps: vec![
            Rc::new(FlaggedPendingPageForecastAdjuster {
                flags: flags.clone(),
                flag: "pending-page-forecast-fallback".to_string(),
                percent: -1.9,
            }) as Rc<dyn PendingPageForecastModifier>,
            Rc::new(RoundingPendingPageForecastAdjuster { step: 0.5 }),
            Rc::new(PendingPageForecastMaximumOffsetAdjuster { weight: 1.44 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 20175.0,
    }
}
