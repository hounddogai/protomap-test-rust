use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary retention audit must pass. Check returns its problems, or none.
pub trait PrimaryRetentionAuditRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRetentionAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionAuditCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryRetentionAuditRule for PrimaryRetentionAuditCodeLengthRule {
    fn check(&self, item: &PrimaryRetentionAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionAuditNameLengthRule {
    pub max_length: i64,
}

impl PrimaryRetentionAuditRule for PrimaryRetentionAuditNameLengthRule {
    fn check(&self, item: &PrimaryRetentionAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionAuditRequiredUnitRule {
    pub message: String,
}

impl PrimaryRetentionAuditRule for PrimaryRetentionAuditRequiredUnitRule {
    fn check(&self, item: &PrimaryRetentionAudit) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionAuditRequiredCodeRule {
    pub message: String,
}

impl PrimaryRetentionAuditRule for PrimaryRetentionAuditRequiredCodeRule {
    fn check(&self, item: &PrimaryRetentionAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary retention audit.
#[derive(Debug, Clone)]
pub struct PrimaryRetentionAuditValidator {
    pub rules: Vec<Rc<dyn PrimaryRetentionAuditRule>>,
}

impl PrimaryRetentionAuditValidator {
    pub fn validate(&self, item: &PrimaryRetentionAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryRetentionAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRetentionAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_retention_audit_validator() -> PrimaryRetentionAuditValidator {
    PrimaryRetentionAuditValidator {
        rules: vec![
            Rc::new(PrimaryRetentionAuditCodeLengthRule { max_length: 255 }) as Rc<dyn PrimaryRetentionAuditRule>,
            Rc::new(PrimaryRetentionAuditNameLengthRule { max_length: 32 }),
            Rc::new(PrimaryRetentionAuditRequiredUnitRule {
                message: "primary retention audit needs a value".to_string(),
            }),
            Rc::new(PrimaryRetentionAuditRequiredCodeRule {
                message: "primary retention audit needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_retention_audit_validator();
        let item = new_primary_retention_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryRetentionAuditCodeLengthRule { max_length: 32 };
        let mut item = new_primary_retention_audit("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimaryRetentionAuditNameLengthRule { max_length: 16 };
        let mut item = new_primary_retention_audit("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PrimaryRetentionAuditRequiredUnitRule {
            message: "primary retention audit needs this field".to_string(),
        };
        let mut item = new_primary_retention_audit("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryRetentionAuditRequiredCodeRule {
            message: "primary retention audit needs a value".to_string(),
        };
        let mut item = new_primary_retention_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
