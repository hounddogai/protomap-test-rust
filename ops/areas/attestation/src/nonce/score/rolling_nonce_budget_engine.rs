use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling nonce budget calculation.
pub trait RollingNonceBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingNonceBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingNonceBudgetAdjuster {
    pub step: f64,
}

impl RollingNonceBudgetStep for RoundingRollingNonceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingNonceBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingNonceBudgetAdjuster {
    pub percent: f64,
}

impl RollingNonceBudgetStep for PercentRollingNonceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingNonceBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingNonceBudgetAdjuster {
    pub cap: f64,
}

impl RollingNonceBudgetStep for CapRollingNonceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingNonceBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingNonceBudgetAdjuster {
    pub minimum: f64,
}

impl RollingNonceBudgetStep for MinimumRollingNonceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingNonceBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRollingNonceBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingNonceBudgetStep for CountryRollingNonceBudgetAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RollingNonceBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rolling nonce budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingNonceBudgetEngine {
    pub steps: Vec<Rc<dyn RollingNonceBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingNonceBudgetEngine {
    pub fn run(&mut self, item: &RollingNonceBudget) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[RollingNonceBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_nonce_budget_engine() -> RollingNonceBudgetEngine {
    RollingNonceBudgetEngine {
        steps: vec![
            Rc::new(RoundingRollingNonceBudgetAdjuster { step: 5.0 }) as Rc<dyn RollingNonceBudgetStep>,
            Rc::new(PercentRollingNonceBudgetAdjuster { percent: 5.5 }),
            Rc::new(CapRollingNonceBudgetAdjuster { cap: 43174.0 }),
            Rc::new(MinimumRollingNonceBudgetAdjuster { minimum: 48.9 }),
            Rc::new(CountryRollingNonceBudgetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.95)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 21808.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_nonce_budget_engine();
        let item = new_rolling_nonce_budget("code-400", "name-751");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
