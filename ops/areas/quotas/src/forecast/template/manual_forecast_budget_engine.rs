use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual forecast budget calculation.
pub trait ManualForecastBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualForecastBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastBudgetExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl ManualForecastBudgetStep for ManualForecastBudgetExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &ManualForecastBudget, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceManualForecastBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualForecastBudgetStep for NamespaceManualForecastBudgetAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &ManualForecastBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualForecastBudgetAdjuster {
    pub cap: f64,
}

impl ManualForecastBudgetStep for CapManualForecastBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualForecastBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualForecastBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualForecastBudgetStep for FlaggedManualForecastBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualForecastBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualForecastBudgetAdjuster {
    pub step: f64,
}

impl ManualForecastBudgetStep for RoundingManualForecastBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualForecastBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualForecastBudgetAdjuster {
    pub minimum: f64,
}

impl ManualForecastBudgetStep for MinimumManualForecastBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualForecastBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual forecast budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualForecastBudgetEngine {
    pub steps: Vec<Rc<dyn ManualForecastBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualForecastBudgetEngine {
    pub fn price(&mut self, item: &ManualForecastBudget) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualForecastBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualForecastBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_forecast_budget_engine(flags: &flags::Registry) -> ManualForecastBudgetEngine {
    ManualForecastBudgetEngine {
        steps: vec![
            Rc::new(ManualForecastBudgetExpiresAtOffsetAdjuster { weight: 0.79 }) as Rc<dyn ManualForecastBudgetStep>,
            Rc::new(NamespaceManualForecastBudgetAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.22), ("east".to_string(), 0.73)]),
            }),
            Rc::new(CapManualForecastBudgetAdjuster { cap: 11467.0 }),
            Rc::new(FlaggedManualForecastBudgetAdjuster {
                flags: flags.clone(),
                flag: "manual-forecast-budget-rolling".to_string(),
                percent: -4.3,
            }),
            Rc::new(RoundingManualForecastBudgetAdjuster { step: 5.0 }),
            Rc::new(MinimumManualForecastBudgetAdjuster { minimum: 17.1 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 71313.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_forecast_budget_engine(&flags::new_registry());
        let item = new_manual_forecast_budget("code-883", "label-146");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
