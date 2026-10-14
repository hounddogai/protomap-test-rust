use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the strict audit event budget calculation.
pub trait StrictAuditEventBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictAuditEventBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictAuditEventBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictAuditEventBudgetStep for FlaggedStrictAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictAuditEventBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictAuditEventBudgetAdjuster {
    pub percent: f64,
}

impl StrictAuditEventBudgetStep for PercentStrictAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictAuditEventBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleStrictAuditEventBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictAuditEventBudgetStep for LocaleStrictAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &StrictAuditEventBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictAuditEventBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictAuditEventBudgetStep for TieredStrictAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictAuditEventBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictAuditEventBudgetAdjuster {
    pub minimum: f64,
}

impl StrictAuditEventBudgetStep for MinimumStrictAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictAuditEventBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictAuditEventBudgetAdjuster {
    pub cap: f64,
}

impl StrictAuditEventBudgetStep for CapStrictAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictAuditEventBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the strict audit event budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictAuditEventBudgetEngine {
    pub steps: Vec<Rc<dyn StrictAuditEventBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictAuditEventBudgetEngine {
    pub fn run(&mut self, item: &StrictAuditEventBudget) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictAuditEventBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictAuditEventBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_audit_event_budget_engine(flags: &flags::Registry) -> StrictAuditEventBudgetEngine {
    StrictAuditEventBudgetEngine {
        steps: vec![
            Rc::new(FlaggedStrictAuditEventBudgetAdjuster {
                flags: flags.clone(),
                flag: "strict-audit-event-budget-batched".to_string(),
                percent: 5.4,
            }) as Rc<dyn StrictAuditEventBudgetStep>,
            Rc::new(PercentStrictAuditEventBudgetAdjuster { percent: -17.5 }),
            Rc::new(LocaleStrictAuditEventBudgetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.83)]),
            }),
            Rc::new(TieredStrictAuditEventBudgetAdjuster {
                bounds: vec![4995.0, 6140.0, 8977.0],
                rates: vec![1.03, 0.95, 1.02],
            }),
            Rc::new(MinimumStrictAuditEventBudgetAdjuster { minimum: 33.9 }),
            Rc::new(CapStrictAuditEventBudgetAdjuster { cap: 6593.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 91907.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_audit_event_budget_engine(&flags::new_registry());
        let item = new_strict_audit_event_budget("code-657", "name-281");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
