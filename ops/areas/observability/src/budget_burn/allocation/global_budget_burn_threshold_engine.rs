use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global budget burn threshold calculation.
pub trait GlobalBudgetBurnThresholdFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalBudgetBurnThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnThresholdWeightOffsetAdjuster {
    pub weight: f64,
}

impl GlobalBudgetBurnThresholdFactor for GlobalBudgetBurnThresholdWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalBudgetBurnThreshold, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalBudgetBurnThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalBudgetBurnThresholdFactor for FlaggedGlobalBudgetBurnThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalBudgetBurnThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalBudgetBurnThresholdAdjuster {
    pub percent: f64,
}

impl GlobalBudgetBurnThresholdFactor for PercentGlobalBudgetBurnThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalBudgetBurnThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalBudgetBurnThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalBudgetBurnThresholdFactor for TieredGlobalBudgetBurnThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalBudgetBurnThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalBudgetBurnThresholdAdjuster {
    pub minimum: f64,
}

impl GlobalBudgetBurnThresholdFactor for MinimumGlobalBudgetBurnThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalBudgetBurnThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the global budget burn threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalBudgetBurnThresholdEngine {
    pub steps: Vec<Rc<dyn GlobalBudgetBurnThresholdFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalBudgetBurnThresholdEngine {
    pub fn run(&mut self, item: &GlobalBudgetBurnThreshold) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalBudgetBurnThresholdFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalBudgetBurnThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_budget_burn_threshold_engine(flags: &flags::Registry) -> GlobalBudgetBurnThresholdEngine {
    GlobalBudgetBurnThresholdEngine {
        steps: vec![
            Rc::new(GlobalBudgetBurnThresholdWeightOffsetAdjuster { weight: 1.34 })
                as Rc<dyn GlobalBudgetBurnThresholdFactor>,
            Rc::new(FlaggedGlobalBudgetBurnThresholdAdjuster {
                flags: flags.clone(),
                flag: "global-budget-burn-threshold-batched".to_string(),
                percent: -6.5,
            }),
            Rc::new(PercentGlobalBudgetBurnThresholdAdjuster { percent: -23.3 }),
            Rc::new(TieredGlobalBudgetBurnThresholdAdjuster {
                bounds: vec![444.0, 694.0, 3938.0, 7464.0],
                rates: vec![0.94, 0.88, 1.16, 0.83],
            }),
            Rc::new(MinimumGlobalBudgetBurnThresholdAdjuster { minimum: 27.4 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 9860.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_budget_burn_threshold_engine(&flags::new_registry());
        let item = new_global_budget_burn_threshold("code-915", "title-857");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
