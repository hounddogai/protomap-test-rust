use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict forecast allocation calculation.
pub trait StrictForecastAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictForecastAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleStrictForecastAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictForecastAllocationModifier for TitleStrictForecastAllocationAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &StrictForecastAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictForecastAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictForecastAllocationModifier for FlaggedStrictForecastAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictForecastAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictForecastAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictForecastAllocationModifier for TieredStrictForecastAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictForecastAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictForecastAllocationAdjuster {
    pub minimum: f64,
}

impl StrictForecastAllocationModifier for MinimumStrictForecastAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictForecastAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastAllocationMarginOffsetAdjuster {
    pub weight: f64,
}

impl StrictForecastAllocationModifier for StrictForecastAllocationMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &StrictForecastAllocation, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the strict forecast allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictForecastAllocationEngine {
    pub steps: Vec<Rc<dyn StrictForecastAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictForecastAllocationEngine {
    pub fn price(&mut self, item: &StrictForecastAllocation) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[StrictForecastAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_forecast_allocation_engine(flags: &flags::Registry) -> StrictForecastAllocationEngine {
    StrictForecastAllocationEngine {
        steps: vec![
            Rc::new(TitleStrictForecastAllocationAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.71),
                    ("online".to_string(), 1.28),
                    ("retail".to_string(), 0.76),
                ]),
            }) as Rc<dyn StrictForecastAllocationModifier>,
            Rc::new(FlaggedStrictForecastAllocationAdjuster {
                flags: flags.clone(),
                flag: "strict-forecast-allocation-manual".to_string(),
                percent: 7.9,
            }),
            Rc::new(TieredStrictForecastAllocationAdjuster {
                bounds: vec![3816.0, 4600.0],
                rates: vec![1.07, 0.94],
            }),
            Rc::new(MinimumStrictForecastAllocationAdjuster { minimum: 31.6 }),
            Rc::new(StrictForecastAllocationMarginOffsetAdjuster { weight: 0.53 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 45667.5,
    }
}
