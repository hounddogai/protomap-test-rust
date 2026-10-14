use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal overage estimate must pass. Check returns its problems, or none.
pub trait SeasonalOverageEstimateRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalOverageEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageEstimateRequiredScopeRule {
    pub message: String,
}

impl SeasonalOverageEstimateRule for SeasonalOverageEstimateRequiredScopeRule {
    fn check(&self, item: &SeasonalOverageEstimate) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageEstimateRequiredCodeRule {
    pub message: String,
}

impl SeasonalOverageEstimateRule for SeasonalOverageEstimateRequiredCodeRule {
    fn check(&self, item: &SeasonalOverageEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageEstimateCodePrefixRule {
    pub prefix: String,
}

impl SeasonalOverageEstimateRule for SeasonalOverageEstimateCodePrefixRule {
    fn check(&self, item: &SeasonalOverageEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal overage estimate.
#[derive(Debug, Clone)]
pub struct SeasonalOverageEstimateValidator {
    pub rules: Vec<Rc<dyn SeasonalOverageEstimateRule>>,
}

impl SeasonalOverageEstimateValidator {
    pub fn validate(&self, item: &SeasonalOverageEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalOverageEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalOverageEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_overage_estimate_validator() -> SeasonalOverageEstimateValidator {
    SeasonalOverageEstimateValidator {
        rules: vec![
            Rc::new(SeasonalOverageEstimateRequiredScopeRule {
                message: "seasonal overage estimate needs this field".to_string(),
            }) as Rc<dyn SeasonalOverageEstimateRule>,
            Rc::new(SeasonalOverageEstimateRequiredCodeRule {
                message: "seasonal overage estimate needs a value".to_string(),
            }),
            Rc::new(SeasonalOverageEstimateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
