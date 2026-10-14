use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the freeze rule calculation.
pub trait FreezeRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeRuleAdjuster {
    pub step: f64,
}

impl FreezeRuleStep for RoundingFreezeRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameFreezeRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeRuleStep for NameFreezeRuleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &FreezeRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFreezeRuleAdjuster {
    pub cap: f64,
}

impl FreezeRuleStep for CapFreezeRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FreezeRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the freeze rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeRuleEngine {
    pub steps: Vec<Rc<dyn FreezeRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeRuleEngine {
    pub fn compute(&mut self, item: &FreezeRule) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezeRuleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FreezeRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_freeze_rule_engine() -> FreezeRuleEngine {
    FreezeRuleEngine {
        steps: vec![
            Rc::new(RoundingFreezeRuleAdjuster { step: 0.5 }) as Rc<dyn FreezeRuleStep>,
            Rc::new(NameFreezeRuleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.33)]),
            }),
            Rc::new(CapFreezeRuleAdjuster { cap: 35484.0 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 72927.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_rule_engine();
        let item = new_freeze_rule("code-224", "name-576");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
