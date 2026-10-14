use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary binding rule calculation.
pub trait PrimaryBindingRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryBindingRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryBindingRuleAdjuster {
    pub cap: f64,
}

impl PrimaryBindingRuleModifier for CapPrimaryBindingRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryBindingRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryBindingRuleAdjuster {
    pub step: f64,
}

impl PrimaryBindingRuleModifier for RoundingPrimaryBindingRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryBindingRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePrimaryBindingRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryBindingRuleModifier for SourcePrimaryBindingRuleAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PrimaryBindingRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryBindingRuleAdjuster {
    pub minimum: f64,
}

impl PrimaryBindingRuleModifier for MinimumPrimaryBindingRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryBindingRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryBindingRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryBindingRuleModifier for FlaggedPrimaryBindingRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryBindingRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary binding rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryBindingRuleEngine {
    pub steps: Vec<Rc<dyn PrimaryBindingRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryBindingRuleEngine {
    pub fn compute(&mut self, item: &PrimaryBindingRule) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryBindingRuleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryBindingRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_binding_rule_engine(flags: &flags::Registry) -> PrimaryBindingRuleEngine {
    PrimaryBindingRuleEngine {
        steps: vec![
            Rc::new(CapPrimaryBindingRuleAdjuster { cap: 21810.0 }) as Rc<dyn PrimaryBindingRuleModifier>,
            Rc::new(RoundingPrimaryBindingRuleAdjuster { step: 5.0 }),
            Rc::new(SourcePrimaryBindingRuleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.23)]),
            }),
            Rc::new(MinimumPrimaryBindingRuleAdjuster { minimum: 16.1 }),
            Rc::new(FlaggedPrimaryBindingRuleAdjuster {
                flags: flags.clone(),
                flag: "primary-binding-rule-weighted".to_string(),
                percent: -3.2,
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 99325.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_binding_rule_engine(&flags::new_registry());
        let item = new_primary_binding_rule("code-897", "label-782");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
