use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback budget burn preference calculation.
pub trait FallbackBudgetBurnPreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackBudgetBurnPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackBudgetBurnPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackBudgetBurnPreferenceFactor for TieredFallbackBudgetBurnPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackBudgetBurnPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackBudgetBurnPreferenceAdjuster {
    pub minimum: f64,
}

impl FallbackBudgetBurnPreferenceFactor for MinimumFallbackBudgetBurnPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackBudgetBurnPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnPreferenceFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl FallbackBudgetBurnPreferenceFactor for FallbackBudgetBurnPreferenceFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackBudgetBurnPreference, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeFallbackBudgetBurnPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackBudgetBurnPreferenceFactor for ScopeFallbackBudgetBurnPreferenceAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &FallbackBudgetBurnPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackBudgetBurnPreferenceAdjuster {
    pub percent: f64,
}

impl FallbackBudgetBurnPreferenceFactor for PercentFallbackBudgetBurnPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackBudgetBurnPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackBudgetBurnPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackBudgetBurnPreferenceFactor for FlaggedFallbackBudgetBurnPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackBudgetBurnPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback budget burn preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackBudgetBurnPreferenceEngine {
    pub steps: Vec<Rc<dyn FallbackBudgetBurnPreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackBudgetBurnPreferenceEngine {
    pub fn evaluate(&mut self, item: &FallbackBudgetBurnPreference) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[FallbackBudgetBurnPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_budget_burn_preference_engine(flags: &flags::Registry) -> FallbackBudgetBurnPreferenceEngine {
    FallbackBudgetBurnPreferenceEngine {
        steps: vec![
            Rc::new(TieredFallbackBudgetBurnPreferenceAdjuster {
                bounds: vec![1908.0, 8652.0, 9762.0],
                rates: vec![0.94, 0.91, 1.17],
            }) as Rc<dyn FallbackBudgetBurnPreferenceFactor>,
            Rc::new(MinimumFallbackBudgetBurnPreferenceAdjuster { minimum: 39.0 }),
            Rc::new(FallbackBudgetBurnPreferenceFloorValueOffsetAdjuster { weight: 1.46 }),
            Rc::new(ScopeFallbackBudgetBurnPreferenceAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.75),
                    ("east".to_string(), 0.78),
                    ("north".to_string(), 0.88),
                ]),
            }),
            Rc::new(PercentFallbackBudgetBurnPreferenceAdjuster { percent: 19.0 }),
            Rc::new(FlaggedFallbackBudgetBurnPreferenceAdjuster {
                flags: flags.clone(),
                flag: "fallback-budget-burn-preference-legacy".to_string(),
                percent: -5.6,
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 80484.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_budget_burn_preference_engine(&flags::new_registry());
        let item = new_fallback_budget_burn_preference("code-529", "name-629");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
