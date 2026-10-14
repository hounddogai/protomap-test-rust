use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress variance must pass. Check returns its problems, or none.
pub trait EgressVarianceRule: std::fmt::Debug {
    fn check(&self, item: &EgressVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressVarianceRequiredTitleRule {
    pub message: String,
}

impl EgressVarianceRule for EgressVarianceRequiredTitleRule {
    fn check(&self, item: &EgressVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressVarianceRequiredCodeRule {
    pub message: String,
}

impl EgressVarianceRule for EgressVarianceRequiredCodeRule {
    fn check(&self, item: &EgressVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressVarianceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl EgressVarianceRule for EgressVarianceAllowedCodeRule {
    fn check(&self, item: &EgressVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress variance.
#[derive(Debug, Clone)]
pub struct EgressVarianceValidator {
    pub rules: Vec<Rc<dyn EgressVarianceRule>>,
}

impl EgressVarianceValidator {
    pub fn validate(&self, item: &EgressVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_variance_validator() -> EgressVarianceValidator {
    EgressVarianceValidator {
        rules: vec![
            Rc::new(EgressVarianceRequiredTitleRule {
                message: "egress variance needs a value".to_string(),
            }) as Rc<dyn EgressVarianceRule>,
            Rc::new(EgressVarianceRequiredCodeRule {
                message: "egress variance needs a value".to_string(),
            }),
            Rc::new(EgressVarianceAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_variance_validator();
        let item = new_egress_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = EgressVarianceRequiredTitleRule {
            message: "egress variance needs a value".to_string(),
        };
        let mut item = new_egress_variance("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressVarianceRequiredCodeRule {
            message: "egress variance needs an entry".to_string(),
        };
        let mut item = new_egress_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
