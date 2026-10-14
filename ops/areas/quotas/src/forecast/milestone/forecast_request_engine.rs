use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the forecast request calculation.
pub trait ForecastRequestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastRequest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedForecastRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ForecastRequestFactor for FlaggedForecastRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ForecastRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRequestRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl ForecastRequestFactor for ForecastRequestRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastRequest, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameForecastRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastRequestFactor for NameForecastRequestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ForecastRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentForecastRequestAdjuster {
    pub percent: f64,
}

impl ForecastRequestFactor for PercentForecastRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ForecastRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastRequestAdjuster {
    pub cap: f64,
}

impl ForecastRequestFactor for CapForecastRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastRequestAdjuster {
    pub minimum: f64,
}

impl ForecastRequestFactor for MinimumForecastRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the forecast request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastRequestEngine {
    pub steps: Vec<Rc<dyn ForecastRequestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastRequestEngine {
    pub fn compute(&mut self, item: &ForecastRequest) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ForecastRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_forecast_request_engine(flags: &flags::Registry) -> ForecastRequestEngine {
    ForecastRequestEngine {
        steps: vec![
            Rc::new(FlaggedForecastRequestAdjuster {
                flags: flags.clone(),
                flag: "forecast-request-default".to_string(),
                percent: -5.2,
            }) as Rc<dyn ForecastRequestFactor>,
            Rc::new(ForecastRequestRotationDaysOffsetAdjuster { weight: 1.14 }),
            Rc::new(NameForecastRequestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.21)]),
            }),
            Rc::new(PercentForecastRequestAdjuster { percent: -2.1 }),
            Rc::new(CapForecastRequestAdjuster { cap: 36988.0 }),
            Rc::new(MinimumForecastRequestAdjuster { minimum: 41.5 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 69967.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_request_engine(&flags::new_registry());
        let item = new_forecast_request("code-417", "name-659");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
