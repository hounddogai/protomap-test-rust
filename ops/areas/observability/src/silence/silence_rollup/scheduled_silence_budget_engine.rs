use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled silence budget calculation.
pub trait ScheduledSilenceBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledSilenceBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameScheduledSilenceBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledSilenceBudgetModifier for NameScheduledSilenceBudgetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ScheduledSilenceBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledSilenceBudgetAdjuster {
    pub step: f64,
}

impl ScheduledSilenceBudgetModifier for RoundingScheduledSilenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledSilenceBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledSilenceBudgetAdjuster {
    pub minimum: f64,
}

impl ScheduledSilenceBudgetModifier for MinimumScheduledSilenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledSilenceBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceBudgetCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledSilenceBudgetModifier for ScheduledSilenceBudgetCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledSilenceBudget, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledSilenceBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledSilenceBudgetModifier for FlaggedScheduledSilenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledSilenceBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled silence budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledSilenceBudgetEngine {
    pub steps: Vec<Rc<dyn ScheduledSilenceBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledSilenceBudgetEngine {
    pub fn price(&mut self, item: &ScheduledSilenceBudget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledSilenceBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledSilenceBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_silence_budget_engine(flags: &flags::Registry) -> ScheduledSilenceBudgetEngine {
    ScheduledSilenceBudgetEngine {
        steps: vec![
            Rc::new(NameScheduledSilenceBudgetAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.07), ("east".to_string(), 0.82)]),
            }) as Rc<dyn ScheduledSilenceBudgetModifier>,
            Rc::new(RoundingScheduledSilenceBudgetAdjuster { step: 0.1 }),
            Rc::new(MinimumScheduledSilenceBudgetAdjuster { minimum: 10.8 }),
            Rc::new(ScheduledSilenceBudgetCreatedAtOffsetAdjuster { weight: 0.41 }),
            Rc::new(FlaggedScheduledSilenceBudgetAdjuster {
                flags: flags.clone(),
                flag: "scheduled-silence-budget-weighted".to_string(),
                percent: 2.7,
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 6237.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_silence_budget_engine(&flags::new_registry());
        let item = new_scheduled_silence_budget("code-918", "name-875");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
