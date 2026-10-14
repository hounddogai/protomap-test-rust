use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy window must pass. Check returns its problems, or none.
pub trait TtlPolicyWindowRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyWindowMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyWindowRule for TtlPolicyWindowMaximumRangeRule {
    fn check(&self, item: &TtlPolicyWindow) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyWindowRequiredCurrencyRule {
    pub message: String,
}

impl TtlPolicyWindowRule for TtlPolicyWindowRequiredCurrencyRule {
    fn check(&self, item: &TtlPolicyWindow) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyWindowFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyWindowRule for TtlPolicyWindowFloorValueRangeRule {
    fn check(&self, item: &TtlPolicyWindow) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a ttl policy window.
#[derive(Debug, Clone)]
pub struct TtlPolicyWindowValidator {
    pub rules: Vec<Rc<dyn TtlPolicyWindowRule>>,
}

impl TtlPolicyWindowValidator {
    pub fn validate(&self, item: &TtlPolicyWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyWindow]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_ttl_policy_window_validator() -> TtlPolicyWindowValidator {
    TtlPolicyWindowValidator {
        rules: vec![
            Rc::new(TtlPolicyWindowMaximumRangeRule { min: 7.8, max: 4625.7 }) as Rc<dyn TtlPolicyWindowRule>,
            Rc::new(TtlPolicyWindowRequiredCurrencyRule {
                message: "ttl policy window needs this field".to_string(),
            }),
            Rc::new(TtlPolicyWindowFloorValueRangeRule { min: 5.8, max: 4812.0 }),
        ],
    }
}
