use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling flow log budget calculation.
pub trait RollingFlowLogBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingFlowLogBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingFlowLogBudgetAdjuster {
    pub cap: f64,
}

impl RollingFlowLogBudgetModifier for CapRollingFlowLogBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingFlowLogBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRollingFlowLogBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingFlowLogBudgetModifier for ChannelRollingFlowLogBudgetAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RollingFlowLogBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingFlowLogBudgetAdjuster {
    pub minimum: f64,
}

impl RollingFlowLogBudgetModifier for MinimumRollingFlowLogBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingFlowLogBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingFlowLogBudgetAdjuster {
    pub step: f64,
}

impl RollingFlowLogBudgetModifier for RoundingRollingFlowLogBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingFlowLogBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingFlowLogBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingFlowLogBudgetModifier for FlaggedRollingFlowLogBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingFlowLogBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingFlowLogBudgetAdjuster {
    pub percent: f64,
}

impl RollingFlowLogBudgetModifier for PercentRollingFlowLogBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingFlowLogBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling flow log budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingFlowLogBudgetEngine {
    pub steps: Vec<Rc<dyn RollingFlowLogBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingFlowLogBudgetEngine {
    pub fn price(&mut self, item: &RollingFlowLogBudget) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingFlowLogBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingFlowLogBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_flow_log_budget_engine(flags: &flags::Registry) -> RollingFlowLogBudgetEngine {
    RollingFlowLogBudgetEngine {
        steps: vec![
            Rc::new(CapRollingFlowLogBudgetAdjuster { cap: 48235.0 }) as Rc<dyn RollingFlowLogBudgetModifier>,
            Rc::new(ChannelRollingFlowLogBudgetAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.76),
                    ("west".to_string(), 1.27),
                    ("central".to_string(), 1.08),
                ]),
            }),
            Rc::new(MinimumRollingFlowLogBudgetAdjuster { minimum: 45.1 }),
            Rc::new(RoundingRollingFlowLogBudgetAdjuster { step: 5.0 }),
            Rc::new(FlaggedRollingFlowLogBudgetAdjuster {
                flags: flags.clone(),
                flag: "rolling-flow-log-budget-tiered".to_string(),
                percent: -8.8,
            }),
            Rc::new(PercentRollingFlowLogBudgetAdjuster { percent: -22.0 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 16397.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_flow_log_budget_engine(&flags::new_registry());
        let item = new_rolling_flow_log_budget("code-436", "title-41");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
