use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted budget burn window calculation.
pub trait WeightedBudgetBurnWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedBudgetBurnWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedBudgetBurnWindowAdjuster {
    pub cap: f64,
}

impl WeightedBudgetBurnWindowStep for CapWeightedBudgetBurnWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedBudgetBurnWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnWindowRatioOffsetAdjuster {
    pub weight: f64,
}

impl WeightedBudgetBurnWindowStep for WeightedBudgetBurnWindowRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedBudgetBurnWindow, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedBudgetBurnWindowAdjuster {
    pub step: f64,
}

impl WeightedBudgetBurnWindowStep for RoundingWeightedBudgetBurnWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedBudgetBurnWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedBudgetBurnWindowAdjuster {
    pub minimum: f64,
}

impl WeightedBudgetBurnWindowStep for MinimumWeightedBudgetBurnWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedBudgetBurnWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedBudgetBurnWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedBudgetBurnWindowStep for FlaggedWeightedBudgetBurnWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedBudgetBurnWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedBudgetBurnWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedBudgetBurnWindowStep for TieredWeightedBudgetBurnWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedBudgetBurnWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted budget burn window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedBudgetBurnWindowEngine {
    pub steps: Vec<Rc<dyn WeightedBudgetBurnWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedBudgetBurnWindowEngine {
    pub fn evaluate(&mut self, item: &WeightedBudgetBurnWindow) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedBudgetBurnWindowStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedBudgetBurnWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_budget_burn_window_engine(flags: &flags::Registry) -> WeightedBudgetBurnWindowEngine {
    WeightedBudgetBurnWindowEngine {
        steps: vec![
            Rc::new(CapWeightedBudgetBurnWindowAdjuster { cap: 26450.0 }) as Rc<dyn WeightedBudgetBurnWindowStep>,
            Rc::new(WeightedBudgetBurnWindowRatioOffsetAdjuster { weight: 0.51 }),
            Rc::new(RoundingWeightedBudgetBurnWindowAdjuster { step: 0.05 }),
            Rc::new(MinimumWeightedBudgetBurnWindowAdjuster { minimum: 10.1 }),
            Rc::new(FlaggedWeightedBudgetBurnWindowAdjuster {
                flags: flags.clone(),
                flag: "weighted-budget-burn-window-lenient".to_string(),
                percent: 1.1,
            }),
            Rc::new(TieredWeightedBudgetBurnWindowAdjuster {
                bounds: vec![4182.0, 5072.0, 7855.0, 9829.0],
                rates: vec![1.14, 0.83, 1.09, 0.94],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 57184.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_budget_burn_window_engine(&flags::new_registry());
        let item = new_weighted_budget_burn_window("code-678", "title-628");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
