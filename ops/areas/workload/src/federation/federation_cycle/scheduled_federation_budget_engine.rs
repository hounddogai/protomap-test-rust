use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled federation budget calculation.
pub trait ScheduledFederationBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledFederationBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleScheduledFederationBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledFederationBudgetFactor for TitleScheduledFederationBudgetAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ScheduledFederationBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledFederationBudgetAdjuster {
    pub step: f64,
}

impl ScheduledFederationBudgetFactor for RoundingScheduledFederationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledFederationBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledFederationBudgetAdjuster {
    pub percent: f64,
}

impl ScheduledFederationBudgetFactor for PercentScheduledFederationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledFederationBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledFederationBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledFederationBudgetFactor for TieredScheduledFederationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledFederationBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledFederationBudgetAdjuster {
    pub cap: f64,
}

impl ScheduledFederationBudgetFactor for CapScheduledFederationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledFederationBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled federation budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledFederationBudgetEngine {
    pub steps: Vec<Rc<dyn ScheduledFederationBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledFederationBudgetEngine {
    pub fn price(&mut self, item: &ScheduledFederationBudget) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledFederationBudgetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledFederationBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_federation_budget_engine() -> ScheduledFederationBudgetEngine {
    ScheduledFederationBudgetEngine {
        steps: vec![
            Rc::new(TitleScheduledFederationBudgetAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.18)]),
            }) as Rc<dyn ScheduledFederationBudgetFactor>,
            Rc::new(RoundingScheduledFederationBudgetAdjuster { step: 5.0 }),
            Rc::new(PercentScheduledFederationBudgetAdjuster { percent: 22.7 }),
            Rc::new(TieredScheduledFederationBudgetAdjuster {
                bounds: vec![6801.0, 9649.0],
                rates: vec![1.13, 1.02],
            }),
            Rc::new(CapScheduledFederationBudgetAdjuster { cap: 8613.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 19214.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_federation_budget_engine();
        let item = new_scheduled_federation_budget("code-315", "title-623");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
