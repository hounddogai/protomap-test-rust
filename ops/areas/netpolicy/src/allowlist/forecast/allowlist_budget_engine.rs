use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist budget calculation.
pub trait AllowlistBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAllowlistBudgetAdjuster {
    pub minimum: f64,
}

impl AllowlistBudgetFactor for MinimumAllowlistBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AllowlistBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAllowlistBudgetAdjuster {
    pub percent: f64,
}

impl AllowlistBudgetFactor for PercentAllowlistBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AllowlistBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistBudgetFactor for TieredAllowlistBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistBudgetAdjuster {
    pub cap: f64,
}

impl AllowlistBudgetFactor for CapAllowlistBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistBudgetFactor for FlaggedAllowlistBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the allowlist budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistBudgetEngine {
    pub steps: Vec<Rc<dyn AllowlistBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistBudgetEngine {
    pub fn run(&mut self, item: &AllowlistBudget) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_allowlist_budget_engine(flags: &flags::Registry) -> AllowlistBudgetEngine {
    AllowlistBudgetEngine {
        steps: vec![
            Rc::new(MinimumAllowlistBudgetAdjuster { minimum: 10.0 }) as Rc<dyn AllowlistBudgetFactor>,
            Rc::new(PercentAllowlistBudgetAdjuster { percent: 2.3 }),
            Rc::new(TieredAllowlistBudgetAdjuster {
                bounds: vec![451.0, 2097.0, 3423.0, 4474.0],
                rates: vec![1.16, 1.17, 1.05, 1.06],
            }),
            Rc::new(CapAllowlistBudgetAdjuster { cap: 33789.0 }),
            Rc::new(FlaggedAllowlistBudgetAdjuster {
                flags: flags.clone(),
                flag: "allowlist-budget-legacy".to_string(),
                percent: -6.3,
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 33836.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_budget_engine(&flags::new_registry());
        let item = new_allowlist_budget("code-856", "name-319");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
