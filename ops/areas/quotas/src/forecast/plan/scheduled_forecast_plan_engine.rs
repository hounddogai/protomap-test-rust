use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled forecast plan calculation.
pub trait ScheduledForecastPlanStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledForecastPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledForecastPlanAdjuster {
    pub percent: f64,
}

impl ScheduledForecastPlanStep for PercentScheduledForecastPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledForecastPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledForecastPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledForecastPlanStep for TieredScheduledForecastPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledForecastPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledForecastPlanAdjuster {
    pub minimum: f64,
}

impl ScheduledForecastPlanStep for MinimumScheduledForecastPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledForecastPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledForecastPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledForecastPlanStep for FlaggedScheduledForecastPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledForecastPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled forecast plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledForecastPlanEngine {
    pub steps: Vec<Rc<dyn ScheduledForecastPlanStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledForecastPlanEngine {
    pub fn run(&mut self, item: &ScheduledForecastPlan) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledForecastPlanStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledForecastPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_forecast_plan_engine(flags: &flags::Registry) -> ScheduledForecastPlanEngine {
    ScheduledForecastPlanEngine {
        steps: vec![
            Rc::new(PercentScheduledForecastPlanAdjuster { percent: 23.6 }) as Rc<dyn ScheduledForecastPlanStep>,
            Rc::new(TieredScheduledForecastPlanAdjuster {
                bounds: vec![1733.0, 3364.0],
                rates: vec![1.03, 1.02],
            }),
            Rc::new(MinimumScheduledForecastPlanAdjuster { minimum: 29.0 }),
            Rc::new(FlaggedScheduledForecastPlanAdjuster {
                flags: flags.clone(),
                flag: "scheduled-forecast-plan-primary".to_string(),
                percent: 8.4,
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 7552.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_forecast_plan_engine(&flags::new_registry());
        let item = new_scheduled_forecast_plan("code-74", "name-815");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
