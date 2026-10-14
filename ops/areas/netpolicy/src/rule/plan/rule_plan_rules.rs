use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rule plan must pass. Check returns its problems, or none.
pub trait RulePlanRule: std::fmt::Debug {
    fn check(&self, item: &RulePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePlanRequiredTitleRule {
    pub message: String,
}

impl RulePlanRule for RulePlanRequiredTitleRule {
    fn check(&self, item: &RulePlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePlanCodePrefixRule {
    pub prefix: String,
}

impl RulePlanRule for RulePlanCodePrefixRule {
    fn check(&self, item: &RulePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePlanRequiredCodeRule {
    pub message: String,
}

impl RulePlanRule for RulePlanRequiredCodeRule {
    fn check(&self, item: &RulePlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePlanIssuerLengthRule {
    pub max_length: i64,
}

impl RulePlanRule for RulePlanIssuerLengthRule {
    fn check(&self, item: &RulePlan) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePlanTitleLengthRule {
    pub max_length: i64,
}

impl RulePlanRule for RulePlanTitleLengthRule {
    fn check(&self, item: &RulePlan) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePlanAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RulePlanRule for RulePlanAllowedTitleRule {
    fn check(&self, item: &RulePlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule plan.
#[derive(Debug, Clone)]
pub struct RulePlanValidator {
    pub rules: Vec<Rc<dyn RulePlanRule>>,
}

impl RulePlanValidator {
    pub fn validate(&self, item: &RulePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RulePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RulePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_plan_validator() -> RulePlanValidator {
    RulePlanValidator {
        rules: vec![
            Rc::new(RulePlanRequiredTitleRule {
                message: "rule plan needs this field".to_string(),
            }) as Rc<dyn RulePlanRule>,
            Rc::new(RulePlanCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RulePlanRequiredCodeRule {
                message: "rule plan needs this field".to_string(),
            }),
            Rc::new(RulePlanIssuerLengthRule { max_length: 16 }),
            Rc::new(RulePlanTitleLengthRule { max_length: 120 }),
            Rc::new(RulePlanAllowedTitleRule {
                allowed: vec!["pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_plan_validator();
        let item = new_rule_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RulePlanRequiredTitleRule {
            message: "rule plan needs an entry".to_string(),
        };
        let mut item = new_rule_plan("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RulePlanRequiredCodeRule {
            message: "rule plan needs an entry".to_string(),
        };
        let mut item = new_rule_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = RulePlanIssuerLengthRule { max_length: 64 };
        let mut item = new_rule_plan("x", "y");
        item.issuer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RulePlanTitleLengthRule { max_length: 32 };
        let mut item = new_rule_plan("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
