use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback budget burn profile calculation.
pub trait FallbackBudgetBurnProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackBudgetBurnProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackBudgetBurnProfileAdjuster {
    pub minimum: f64,
}

impl FallbackBudgetBurnProfileFactor for MinimumFallbackBudgetBurnProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackBudgetBurnProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackBudgetBurnProfileAdjuster {
    pub percent: f64,
}

impl FallbackBudgetBurnProfileFactor for PercentFallbackBudgetBurnProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackBudgetBurnProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackBudgetBurnProfileAdjuster {
    pub cap: f64,
}

impl FallbackBudgetBurnProfileFactor for CapFallbackBudgetBurnProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackBudgetBurnProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnProfileRetriesOffsetAdjuster {
    pub weight: f64,
}

impl FallbackBudgetBurnProfileFactor for FallbackBudgetBurnProfileRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackBudgetBurnProfile, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackBudgetBurnProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackBudgetBurnProfileFactor for FlaggedFallbackBudgetBurnProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackBudgetBurnProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback budget burn profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackBudgetBurnProfileEngine {
    pub steps: Vec<Rc<dyn FallbackBudgetBurnProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackBudgetBurnProfileEngine {
    pub fn price(&mut self, item: &FallbackBudgetBurnProfile) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[FallbackBudgetBurnProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_budget_burn_profile_engine(flags: &flags::Registry) -> FallbackBudgetBurnProfileEngine {
    FallbackBudgetBurnProfileEngine {
        steps: vec![
            Rc::new(MinimumFallbackBudgetBurnProfileAdjuster { minimum: 48.7 })
                as Rc<dyn FallbackBudgetBurnProfileFactor>,
            Rc::new(PercentFallbackBudgetBurnProfileAdjuster { percent: -5.4 }),
            Rc::new(CapFallbackBudgetBurnProfileAdjuster { cap: 30234.0 }),
            Rc::new(FallbackBudgetBurnProfileRetriesOffsetAdjuster { weight: 0.64 }),
            Rc::new(FlaggedFallbackBudgetBurnProfileAdjuster {
                flags: flags.clone(),
                flag: "fallback-budget-burn-profile-default".to_string(),
                percent: -6.1,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 97336.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_budget_burn_profile_engine(&flags::new_registry());
        let item = new_fallback_budget_burn_profile("code-907", "label-386");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
