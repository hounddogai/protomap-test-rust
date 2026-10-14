use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the suspension rule calculation.
pub trait SuspensionRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionRuleAdjuster {
    pub minimum: f64,
}

impl SuspensionRuleStep for MinimumSuspensionRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionRuleAdjuster {
    pub cap: f64,
}

impl SuspensionRuleStep for CapSuspensionRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSuspensionRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionRuleStep for NameSuspensionRuleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SuspensionRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRuleGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionRuleStep for SuspensionRuleGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionRule, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the suspension rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionRuleEngine {
    pub steps: Vec<Rc<dyn SuspensionRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionRuleEngine {
    pub fn evaluate(&mut self, item: &SuspensionRule) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionRuleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_suspension_rule_engine() -> SuspensionRuleEngine {
    SuspensionRuleEngine {
        steps: vec![
            Rc::new(MinimumSuspensionRuleAdjuster { minimum: 44.5 }) as Rc<dyn SuspensionRuleStep>,
            Rc::new(CapSuspensionRuleAdjuster { cap: 30547.0 }),
            Rc::new(NameSuspensionRuleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.94)]),
            }),
            Rc::new(SuspensionRuleGraceDaysOffsetAdjuster { weight: 0.52 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 60481.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_rule_engine();
        let item = new_suspension_rule("code-704", "name-225");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
