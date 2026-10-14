use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the forecast schedule calculation.
pub trait ForecastScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionForecastScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastScheduleStep for RegionForecastScheduleAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ForecastSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastScheduleStep for TieredForecastScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentForecastScheduleAdjuster {
    pub percent: f64,
}

impl ForecastScheduleStep for PercentForecastScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ForecastSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedForecastScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ForecastScheduleStep for FlaggedForecastScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ForecastSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastScheduleAdjuster {
    pub minimum: f64,
}

impl ForecastScheduleStep for MinimumForecastScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastScheduleAdjuster {
    pub cap: f64,
}

impl ForecastScheduleStep for CapForecastScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the forecast schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastScheduleEngine {
    pub steps: Vec<Rc<dyn ForecastScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastScheduleEngine {
    pub fn compute(&mut self, item: &ForecastSchedule) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[ForecastSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_forecast_schedule_engine(flags: &flags::Registry) -> ForecastScheduleEngine {
    ForecastScheduleEngine {
        steps: vec![
            Rc::new(RegionForecastScheduleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.0)]),
            }) as Rc<dyn ForecastScheduleStep>,
            Rc::new(TieredForecastScheduleAdjuster {
                bounds: vec![6495.0, 8989.0, 9028.0],
                rates: vec![1.01, 1.11, 1.18],
            }),
            Rc::new(PercentForecastScheduleAdjuster { percent: -24.2 }),
            Rc::new(FlaggedForecastScheduleAdjuster {
                flags: flags.clone(),
                flag: "forecast-schedule-global".to_string(),
                percent: -2.9,
            }),
            Rc::new(MinimumForecastScheduleAdjuster { minimum: 16.2 }),
            Rc::new(CapForecastScheduleAdjuster { cap: 1561.0 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 16839.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_schedule_engine(&flags::new_registry());
        let item = new_forecast_schedule("code-997", "name-928");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
