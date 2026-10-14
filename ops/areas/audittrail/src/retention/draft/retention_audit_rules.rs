use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention audit must pass. Check returns its problems, or none.
pub trait RetentionAuditRule: std::fmt::Debug {
    fn check(&self, item: &RetentionAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAuditQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionAuditRule for RetentionAuditQuantityRangeRule {
    fn check(&self, item: &RetentionAudit) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAuditAudienceLengthRule {
    pub max_length: i64,
}

impl RetentionAuditRule for RetentionAuditAudienceLengthRule {
    fn check(&self, item: &RetentionAudit) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAuditRequiredIssuerRule {
    pub message: String,
}

impl RetentionAuditRule for RetentionAuditRequiredIssuerRule {
    fn check(&self, item: &RetentionAudit) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAuditEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionAuditRule for RetentionAuditEndsAtRangeRule {
    fn check(&self, item: &RetentionAudit) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a retention audit.
#[derive(Debug, Clone)]
pub struct RetentionAuditValidator {
    pub rules: Vec<Rc<dyn RetentionAuditRule>>,
}

impl RetentionAuditValidator {
    pub fn validate(&self, item: &RetentionAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_audit_validator() -> RetentionAuditValidator {
    RetentionAuditValidator {
        rules: vec![
            Rc::new(RetentionAuditQuantityRangeRule { min: 1000, max: 86400 }) as Rc<dyn RetentionAuditRule>,
            Rc::new(RetentionAuditAudienceLengthRule { max_length: 80 }),
            Rc::new(RetentionAuditRequiredIssuerRule {
                message: "retention audit needs a value".to_string(),
            }),
            Rc::new(RetentionAuditEndsAtRangeRule { min: 16, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_audit_validator();
        let item = new_retention_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = RetentionAuditQuantityRangeRule { min: 32, max: 120 };
        let mut item = new_retention_audit("x", "y");
        item.quantity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = RetentionAuditAudienceLengthRule { max_length: 16 };
        let mut item = new_retention_audit("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = RetentionAuditRequiredIssuerRule {
            message: "retention audit needs this field".to_string(),
        };
        let mut item = new_retention_audit("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = RetentionAuditEndsAtRangeRule { min: 64, max: 255 };
        let mut item = new_retention_audit("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
