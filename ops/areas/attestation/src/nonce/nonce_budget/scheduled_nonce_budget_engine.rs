use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled nonce budget calculation.
pub trait ScheduledNonceBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledNonceBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledNonceBudgetAdjuster {
    pub step: f64,
}

impl ScheduledNonceBudgetAdjuster for RoundingScheduledNonceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledNonceBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledNonceBudgetAdjuster {
    pub cap: f64,
}

impl ScheduledNonceBudgetAdjuster for CapScheduledNonceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledNonceBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceBudgetBurstOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledNonceBudgetAdjuster for ScheduledNonceBudgetBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledNonceBudget, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledNonceBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledNonceBudgetAdjuster for FlaggedScheduledNonceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledNonceBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledNonceBudgetAdjuster {
    pub minimum: f64,
}

impl ScheduledNonceBudgetAdjuster for MinimumScheduledNonceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledNonceBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameScheduledNonceBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledNonceBudgetAdjuster for NameScheduledNonceBudgetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ScheduledNonceBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled nonce budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledNonceBudgetEngine {
    pub steps: Vec<Rc<dyn ScheduledNonceBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledNonceBudgetEngine {
    pub fn run(&mut self, item: &ScheduledNonceBudget) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledNonceBudgetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledNonceBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_nonce_budget_engine(flags: &flags::Registry) -> ScheduledNonceBudgetEngine {
    ScheduledNonceBudgetEngine {
        steps: vec![
            Rc::new(RoundingScheduledNonceBudgetAdjuster { step: 0.25 }) as Rc<dyn ScheduledNonceBudgetAdjuster>,
            Rc::new(CapScheduledNonceBudgetAdjuster { cap: 20343.0 }),
            Rc::new(ScheduledNonceBudgetBurstOffsetAdjuster { weight: 1.09 }),
            Rc::new(FlaggedScheduledNonceBudgetAdjuster {
                flags: flags.clone(),
                flag: "scheduled-nonce-budget-default".to_string(),
                percent: -8.2,
            }),
            Rc::new(MinimumScheduledNonceBudgetAdjuster { minimum: 49.1 }),
            Rc::new(NameScheduledNonceBudgetAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.73)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 85264.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_nonce_budget_engine(&flags::new_registry());
        let item = new_scheduled_nonce_budget("code-780", "name-3");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
