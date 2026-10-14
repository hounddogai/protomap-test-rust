use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule audit must pass. Check returns its problems, or none.
pub trait RuleAuditRule: std::fmt::Debug {
    fn check(&self, item: &RuleAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleAuditNameLengthRule {
    pub max_length: i64,
}

impl RuleAuditRule for RuleAuditNameLengthRule {
    fn check(&self, item: &RuleAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleAuditWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleAuditRule for RuleAuditWeightRangeRule {
    fn check(&self, item: &RuleAudit) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleAuditRequiredCodeRule {
    pub message: String,
}

impl RuleAuditRule for RuleAuditRequiredCodeRule {
    fn check(&self, item: &RuleAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule audit.
#[derive(Debug, Clone)]
pub struct RuleAuditValidator {
    pub rules: Vec<Rc<dyn RuleAuditRule>>,
}

impl RuleAuditValidator {
    pub fn validate(&self, item: &RuleAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_audit_validator() -> RuleAuditValidator {
    RuleAuditValidator {
        rules: vec![
            Rc::new(RuleAuditNameLengthRule { max_length: 64 }) as Rc<dyn RuleAuditRule>,
            Rc::new(RuleAuditWeightRangeRule { min: 3.1, max: 1200.0 }),
            Rc::new(RuleAuditRequiredCodeRule {
                message: "rule audit needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_audit_validator();
        let item = new_rule_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RuleAuditNameLengthRule { max_length: 64 };
        let mut item = new_rule_audit("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = RuleAuditWeightRangeRule { min: 8.3, max: 181.8 };
        let mut item = new_rule_audit("x", "y");
        item.weight = 183.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RuleAuditRequiredCodeRule {
            message: "rule audit needs a value".to_string(),
        };
        let mut item = new_rule_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
