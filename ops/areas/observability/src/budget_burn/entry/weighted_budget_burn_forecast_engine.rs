use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted budget burn forecast calculation.
pub trait WeightedBudgetBurnForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedBudgetBurnForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedBudgetBurnForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedBudgetBurnForecastAdjuster for TieredWeightedBudgetBurnForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedBudgetBurnForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceWeightedBudgetBurnForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedBudgetBurnForecastAdjuster for NamespaceWeightedBudgetBurnForecastAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &WeightedBudgetBurnForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedBudgetBurnForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedBudgetBurnForecastAdjuster for FlaggedWeightedBudgetBurnForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedBudgetBurnForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedBudgetBurnForecastAdjuster {
    pub percent: f64,
}

impl WeightedBudgetBurnForecastAdjuster for PercentWeightedBudgetBurnForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedBudgetBurnForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedBudgetBurnForecastAdjuster {
    pub minimum: f64,
}

impl WeightedBudgetBurnForecastAdjuster for MinimumWeightedBudgetBurnForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedBudgetBurnForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedBudgetBurnForecastAdjuster {
    pub cap: f64,
}

impl WeightedBudgetBurnForecastAdjuster for CapWeightedBudgetBurnForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedBudgetBurnForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the weighted budget burn forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedBudgetBurnForecastEngine {
    pub steps: Vec<Rc<dyn WeightedBudgetBurnForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedBudgetBurnForecastEngine {
    pub fn run(&mut self, item: &WeightedBudgetBurnForecast) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedBudgetBurnForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedBudgetBurnForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_weighted_budget_burn_forecast_engine(flags: &flags::Registry) -> WeightedBudgetBurnForecastEngine {
    WeightedBudgetBurnForecastEngine {
        steps: vec![
            Rc::new(TieredWeightedBudgetBurnForecastAdjuster {
                bounds: vec![3841.0, 6131.0],
                rates: vec![0.86, 0.95],
            }) as Rc<dyn WeightedBudgetBurnForecastAdjuster>,
            Rc::new(NamespaceWeightedBudgetBurnForecastAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.23),
                    ("online".to_string(), 1.07),
                    ("west".to_string(), 0.92),
                ]),
            }),
            Rc::new(FlaggedWeightedBudgetBurnForecastAdjuster {
                flags: flags.clone(),
                flag: "weighted-budget-burn-forecast-tiered".to_string(),
                percent: -6.2,
            }),
            Rc::new(PercentWeightedBudgetBurnForecastAdjuster { percent: -8.8 }),
            Rc::new(MinimumWeightedBudgetBurnForecastAdjuster { minimum: 35.4 }),
            Rc::new(CapWeightedBudgetBurnForecastAdjuster { cap: 9522.0 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 98663.4,
    }
}
