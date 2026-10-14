use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the manual timeline forecast calculation.
pub trait ManualTimelineForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualTimelineForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualTimelineForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualTimelineForecastModifier for TieredManualTimelineForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualTimelineForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTimelineForecastFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl ManualTimelineForecastModifier for ManualTimelineForecastFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &ManualTimelineForecast, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleManualTimelineForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualTimelineForecastModifier for LocaleManualTimelineForecastAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &ManualTimelineForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the manual timeline forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualTimelineForecastEngine {
    pub steps: Vec<Rc<dyn ManualTimelineForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualTimelineForecastEngine {
    pub fn run(&mut self, item: &ManualTimelineForecast) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualTimelineForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualTimelineForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_timeline_forecast_engine() -> ManualTimelineForecastEngine {
    ManualTimelineForecastEngine {
        steps: vec![
            Rc::new(TieredManualTimelineForecastAdjuster {
                bounds: vec![1418.0, 5369.0, 7171.0, 7315.0],
                rates: vec![0.93, 1.14, 1.08, 0.84],
            }) as Rc<dyn ManualTimelineForecastModifier>,
            Rc::new(ManualTimelineForecastFloorValueOffsetAdjuster { weight: 1.55 }),
            Rc::new(LocaleManualTimelineForecastAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.84)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 82262.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_timeline_forecast_engine();
        let item = new_manual_timeline_forecast("code-488", "title-933");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
