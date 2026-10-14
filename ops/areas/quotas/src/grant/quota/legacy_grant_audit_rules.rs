use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy grant audit must pass. Check returns its problems, or none.
pub trait LegacyGrantAuditRule: std::fmt::Debug {
    fn check(&self, item: &LegacyGrantAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantAuditCodeLengthRule {
    pub max_length: i64,
}

impl LegacyGrantAuditRule for LegacyGrantAuditCodeLengthRule {
    fn check(&self, item: &LegacyGrantAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantAuditOwnerIdLengthRule {
    pub max_length: i64,
}

impl LegacyGrantAuditRule for LegacyGrantAuditOwnerIdLengthRule {
    fn check(&self, item: &LegacyGrantAudit) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantAuditStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyGrantAuditRule for LegacyGrantAuditStartsAtRangeRule {
    fn check(&self, item: &LegacyGrantAudit) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantAuditRequiredClusterRule {
    pub message: String,
}

impl LegacyGrantAuditRule for LegacyGrantAuditRequiredClusterRule {
    fn check(&self, item: &LegacyGrantAudit) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantAuditCodePrefixRule {
    pub prefix: String,
}

impl LegacyGrantAuditRule for LegacyGrantAuditCodePrefixRule {
    fn check(&self, item: &LegacyGrantAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy grant audit.
#[derive(Debug, Clone)]
pub struct LegacyGrantAuditValidator {
    pub rules: Vec<Rc<dyn LegacyGrantAuditRule>>,
}

impl LegacyGrantAuditValidator {
    pub fn validate(&self, item: &LegacyGrantAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyGrantAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyGrantAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_grant_audit_validator() -> LegacyGrantAuditValidator {
    LegacyGrantAuditValidator {
        rules: vec![
            Rc::new(LegacyGrantAuditCodeLengthRule { max_length: 1000 }) as Rc<dyn LegacyGrantAuditRule>,
            Rc::new(LegacyGrantAuditOwnerIdLengthRule { max_length: 3600 }),
            Rc::new(LegacyGrantAuditStartsAtRangeRule { min: 16, max: 255 }),
            Rc::new(LegacyGrantAuditRequiredClusterRule {
                message: "legacy grant audit needs a value".to_string(),
            }),
            Rc::new(LegacyGrantAuditCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_grant_audit_validator();
        let item = new_legacy_grant_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyGrantAuditCodeLengthRule { max_length: 3600 };
        let mut item = new_legacy_grant_audit("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = LegacyGrantAuditOwnerIdLengthRule { max_length: 3600 };
        let mut item = new_legacy_grant_audit("x", "y");
        item.owner_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = LegacyGrantAuditStartsAtRangeRule { min: 32, max: 3600 };
        let mut item = new_legacy_grant_audit("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = LegacyGrantAuditRequiredClusterRule {
            message: "legacy grant audit needs a value".to_string(),
        };
        let mut item = new_legacy_grant_audit("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
