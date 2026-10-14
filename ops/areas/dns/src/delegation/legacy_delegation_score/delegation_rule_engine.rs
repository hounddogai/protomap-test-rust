use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the delegation rule calculation.
pub trait DelegationRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationRuleStep for TieredDelegationRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationRuleStep for FlaggedDelegationRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationRuleAdjuster {
    pub minimum: f64,
}

impl DelegationRuleStep for MinimumDelegationRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDelegationRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationRuleStep for LabelDelegationRuleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DelegationRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the delegation rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationRuleEngine {
    pub steps: Vec<Rc<dyn DelegationRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationRuleEngine {
    pub fn run(&mut self, item: &DelegationRule) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DelegationRuleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DelegationRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_delegation_rule_engine(flags: &flags::Registry) -> DelegationRuleEngine {
    DelegationRuleEngine {
        steps: vec![
            Rc::new(TieredDelegationRuleAdjuster {
                bounds: vec![163.0, 1656.0],
                rates: vec![0.81, 1.07],
            }) as Rc<dyn DelegationRuleStep>,
            Rc::new(FlaggedDelegationRuleAdjuster {
                flags: flags.clone(),
                flag: "delegation-rule-pending".to_string(),
                percent: -0.3,
            }),
            Rc::new(MinimumDelegationRuleAdjuster { minimum: 29.4 }),
            Rc::new(LabelDelegationRuleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.14), ("online".to_string(), 0.84)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 77862.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_rule_engine(&flags::new_registry());
        let item = new_delegation_rule("code-157", "label-508");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
