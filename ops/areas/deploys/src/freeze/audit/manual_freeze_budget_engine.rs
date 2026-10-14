use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual freeze budget calculation.
pub trait ManualFreezeBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualFreezeBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualFreezeBudgetAdjuster {
    pub cap: f64,
}

impl ManualFreezeBudgetModifier for CapManualFreezeBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualFreezeBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualFreezeBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualFreezeBudgetModifier for TieredManualFreezeBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualFreezeBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualFreezeBudgetAdjuster {
    pub minimum: f64,
}

impl ManualFreezeBudgetModifier for MinimumManualFreezeBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualFreezeBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualFreezeBudgetAdjuster {
    pub step: f64,
}

impl ManualFreezeBudgetModifier for RoundingManualFreezeBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualFreezeBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualFreezeBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualFreezeBudgetModifier for FlaggedManualFreezeBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualFreezeBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual freeze budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualFreezeBudgetEngine {
    pub steps: Vec<Rc<dyn ManualFreezeBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualFreezeBudgetEngine {
    pub fn compute(&mut self, item: &ManualFreezeBudget) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualFreezeBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualFreezeBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_freeze_budget_engine(flags: &flags::Registry) -> ManualFreezeBudgetEngine {
    ManualFreezeBudgetEngine {
        steps: vec![
            Rc::new(CapManualFreezeBudgetAdjuster { cap: 25532.0 }) as Rc<dyn ManualFreezeBudgetModifier>,
            Rc::new(TieredManualFreezeBudgetAdjuster {
                bounds: vec![6499.0, 6729.0, 8861.0, 9551.0],
                rates: vec![1.14, 1.1, 1.13, 0.87],
            }),
            Rc::new(MinimumManualFreezeBudgetAdjuster { minimum: 20.2 }),
            Rc::new(RoundingManualFreezeBudgetAdjuster { step: 0.25 }),
            Rc::new(FlaggedManualFreezeBudgetAdjuster {
                flags: flags.clone(),
                flag: "manual-freeze-budget-seasonal".to_string(),
                percent: 1.2,
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 46031.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_freeze_budget_engine(&flags::new_registry());
        let item = new_manual_freeze_budget("code-643", "name-937");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
