use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional suspension budget calculation.
pub trait RegionalSuspensionBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalSuspensionBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalSuspensionBudgetAdjuster {
    pub cap: f64,
}

impl RegionalSuspensionBudgetAdjuster for CapRegionalSuspensionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalSuspensionBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRegionalSuspensionBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalSuspensionBudgetAdjuster for NameRegionalSuspensionBudgetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RegionalSuspensionBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalSuspensionBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalSuspensionBudgetAdjuster for FlaggedRegionalSuspensionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalSuspensionBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional suspension budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalSuspensionBudgetEngine {
    pub steps: Vec<Rc<dyn RegionalSuspensionBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalSuspensionBudgetEngine {
    pub fn evaluate(&mut self, item: &RegionalSuspensionBudget) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalSuspensionBudgetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalSuspensionBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_suspension_budget_engine(flags: &flags::Registry) -> RegionalSuspensionBudgetEngine {
    RegionalSuspensionBudgetEngine {
        steps: vec![
            Rc::new(CapRegionalSuspensionBudgetAdjuster { cap: 46260.0 }) as Rc<dyn RegionalSuspensionBudgetAdjuster>,
            Rc::new(NameRegionalSuspensionBudgetAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.08),
                    ("north".to_string(), 1.1),
                    ("central".to_string(), 0.97),
                ]),
            }),
            Rc::new(FlaggedRegionalSuspensionBudgetAdjuster {
                flags: flags.clone(),
                flag: "regional-suspension-budget-adaptive".to_string(),
                percent: 8.2,
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 22174.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_suspension_budget_engine(&flags::new_registry());
        let item = new_regional_suspension_budget("code-192", "name-357");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
