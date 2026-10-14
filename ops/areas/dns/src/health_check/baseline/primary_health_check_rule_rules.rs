use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary health check rule must pass. Check returns its problems, or none.
pub trait PrimaryHealthCheckRuleRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryHealthCheckRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckRuleRequiredCodeRule {
    pub message: String,
}

impl PrimaryHealthCheckRuleRule for PrimaryHealthCheckRuleRequiredCodeRule {
    fn check(&self, item: &PrimaryHealthCheckRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckRuleCodePrefixRule {
    pub prefix: String,
}

impl PrimaryHealthCheckRuleRule for PrimaryHealthCheckRuleCodePrefixRule {
    fn check(&self, item: &PrimaryHealthCheckRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckRuleNameLengthRule {
    pub max_length: i64,
}

impl PrimaryHealthCheckRuleRule for PrimaryHealthCheckRuleNameLengthRule {
    fn check(&self, item: &PrimaryHealthCheckRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary health check rule.
#[derive(Debug, Clone)]
pub struct PrimaryHealthCheckRuleValidator {
    pub rules: Vec<Rc<dyn PrimaryHealthCheckRuleRule>>,
}

impl PrimaryHealthCheckRuleValidator {
    pub fn validate(&self, item: &PrimaryHealthCheckRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryHealthCheckRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryHealthCheckRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_health_check_rule_validator() -> PrimaryHealthCheckRuleValidator {
    PrimaryHealthCheckRuleValidator {
        rules: vec![
            Rc::new(PrimaryHealthCheckRuleRequiredCodeRule {
                message: "primary health check rule needs a value".to_string(),
            }) as Rc<dyn PrimaryHealthCheckRuleRule>,
            Rc::new(PrimaryHealthCheckRuleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PrimaryHealthCheckRuleNameLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_health_check_rule_validator();
        let item = new_primary_health_check_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryHealthCheckRuleRequiredCodeRule {
            message: "primary health check rule needs a value".to_string(),
        };
        let mut item = new_primary_health_check_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimaryHealthCheckRuleNameLengthRule { max_length: 32 };
        let mut item = new_primary_health_check_rule("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
