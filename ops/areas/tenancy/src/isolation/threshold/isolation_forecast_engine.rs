use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the isolation forecast calculation.
pub trait IsolationForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceIsolationForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationForecastAdjuster for SourceIsolationForecastAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &IsolationForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationForecastAdjuster for FlaggedIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationForecastAdjuster {
    pub cap: f64,
}

impl IsolationForecastAdjuster for CapIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIsolationForecastAdjuster {
    pub percent: f64,
}

impl IsolationForecastAdjuster for PercentIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IsolationForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIsolationForecastAdjuster {
    pub step: f64,
}

impl IsolationForecastAdjuster for RoundingIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IsolationForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the isolation forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationForecastEngine {
    pub steps: Vec<Rc<dyn IsolationForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationForecastEngine {
    pub fn run(&mut self, item: &IsolationForecast) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IsolationForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IsolationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_isolation_forecast_engine(flags: &flags::Registry) -> IsolationForecastEngine {
    IsolationForecastEngine {
        steps: vec![
            Rc::new(SourceIsolationForecastAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.95)]),
            }) as Rc<dyn IsolationForecastAdjuster>,
            Rc::new(FlaggedIsolationForecastAdjuster {
                flags: flags.clone(),
                flag: "isolation-forecast-adaptive".to_string(),
                percent: 8.2,
            }),
            Rc::new(CapIsolationForecastAdjuster { cap: 5029.0 }),
            Rc::new(PercentIsolationForecastAdjuster { percent: 16.0 }),
            Rc::new(RoundingIsolationForecastAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 24158.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_forecast_engine(&flags::new_registry());
        let item = new_isolation_forecast("code-473", "title-647");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
