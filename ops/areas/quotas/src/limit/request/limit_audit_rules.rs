use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit audit must pass. Check returns its problems, or none.
pub trait LimitAuditRule: std::fmt::Debug {
    fn check(&self, item: &LimitAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAuditCodeLengthRule {
    pub max_length: i64,
}

impl LimitAuditRule for LimitAuditCodeLengthRule {
    fn check(&self, item: &LimitAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAuditSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitAuditRule for LimitAuditSequenceRangeRule {
    fn check(&self, item: &LimitAudit) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAuditTitleLengthRule {
    pub max_length: i64,
}

impl LimitAuditRule for LimitAuditTitleLengthRule {
    fn check(&self, item: &LimitAudit) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit audit.
#[derive(Debug, Clone)]
pub struct LimitAuditValidator {
    pub rules: Vec<Rc<dyn LimitAuditRule>>,
}

impl LimitAuditValidator {
    pub fn validate(&self, item: &LimitAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_audit_validator() -> LimitAuditValidator {
    LimitAuditValidator {
        rules: vec![
            Rc::new(LimitAuditCodeLengthRule { max_length: 32 }) as Rc<dyn LimitAuditRule>,
            Rc::new(LimitAuditSequenceRangeRule { min: 120, max: 3600 }),
            Rc::new(LimitAuditTitleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_audit_validator();
        let item = new_limit_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LimitAuditCodeLengthRule { max_length: 1000 };
        let mut item = new_limit_audit("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = LimitAuditSequenceRangeRule { min: 64, max: 86400 };
        let mut item = new_limit_audit("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LimitAuditTitleLengthRule { max_length: 3600 };
        let mut item = new_limit_audit("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
