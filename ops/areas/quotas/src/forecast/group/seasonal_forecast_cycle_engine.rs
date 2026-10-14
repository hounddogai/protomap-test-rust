use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal forecast cycle calculation.
pub trait SeasonalForecastCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalForecastCycle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalForecastCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalForecastCycleFactor for FlaggedSeasonalForecastCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalForecastCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalForecastCycleAdjuster {
    pub percent: f64,
}

impl SeasonalForecastCycleFactor for PercentSeasonalForecastCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalForecastCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalForecastCycleAdjuster {
    pub cap: f64,
}

impl SeasonalForecastCycleFactor for CapSeasonalForecastCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalForecastCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceSeasonalForecastCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalForecastCycleFactor for NamespaceSeasonalForecastCycleAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &SeasonalForecastCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal forecast cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalForecastCycleEngine {
    pub steps: Vec<Rc<dyn SeasonalForecastCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalForecastCycleEngine {
    pub fn evaluate(&mut self, item: &SeasonalForecastCycle) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalForecastCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_forecast_cycle_engine(flags: &flags::Registry) -> SeasonalForecastCycleEngine {
    SeasonalForecastCycleEngine {
        steps: vec![
            Rc::new(FlaggedSeasonalForecastCycleAdjuster {
                flags: flags.clone(),
                flag: "seasonal-forecast-cycle-primary".to_string(),
                percent: 6.4,
            }) as Rc<dyn SeasonalForecastCycleFactor>,
            Rc::new(PercentSeasonalForecastCycleAdjuster { percent: -21.2 }),
            Rc::new(CapSeasonalForecastCycleAdjuster { cap: 11279.0 }),
            Rc::new(NamespaceSeasonalForecastCycleAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.29),
                    ("east".to_string(), 1.0),
                    ("north".to_string(), 1.32),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 89385.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_forecast_cycle_engine(&flags::new_registry());
        let item = new_seasonal_forecast_cycle("code-825", "label-597");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
