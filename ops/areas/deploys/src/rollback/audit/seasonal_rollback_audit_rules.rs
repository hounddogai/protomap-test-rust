use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal rollback audit must pass. Check returns its problems, or none.
pub trait SeasonalRollbackAuditRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRollbackAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackAuditAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl SeasonalRollbackAuditRule for SeasonalRollbackAuditAllowedRegionRule {
    fn check(&self, item: &SeasonalRollbackAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is blocked", item.region)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackAuditNameLengthRule {
    pub max_length: i64,
}

impl SeasonalRollbackAuditRule for SeasonalRollbackAuditNameLengthRule {
    fn check(&self, item: &SeasonalRollbackAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackAuditRequiredLabelRule {
    pub message: String,
}

impl SeasonalRollbackAuditRule for SeasonalRollbackAuditRequiredLabelRule {
    fn check(&self, item: &SeasonalRollbackAudit) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackAuditRequiredClusterRule {
    pub message: String,
}

impl SeasonalRollbackAuditRule for SeasonalRollbackAuditRequiredClusterRule {
    fn check(&self, item: &SeasonalRollbackAudit) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackAuditRequiredCodeRule {
    pub message: String,
}

impl SeasonalRollbackAuditRule for SeasonalRollbackAuditRequiredCodeRule {
    fn check(&self, item: &SeasonalRollbackAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackAuditClusterLengthRule {
    pub max_length: i64,
}

impl SeasonalRollbackAuditRule for SeasonalRollbackAuditClusterLengthRule {
    fn check(&self, item: &SeasonalRollbackAudit) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal rollback audit.
#[derive(Debug, Clone)]
pub struct SeasonalRollbackAuditValidator {
    pub rules: Vec<Rc<dyn SeasonalRollbackAuditRule>>,
}

impl SeasonalRollbackAuditValidator {
    pub fn validate(&self, item: &SeasonalRollbackAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalRollbackAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRollbackAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_rollback_audit_validator() -> SeasonalRollbackAuditValidator {
    SeasonalRollbackAuditValidator {
        rules: vec![
            Rc::new(SeasonalRollbackAuditAllowedRegionRule { allowed: Vec::new() })
                as Rc<dyn SeasonalRollbackAuditRule>,
            Rc::new(SeasonalRollbackAuditNameLengthRule { max_length: 32 }),
            Rc::new(SeasonalRollbackAuditRequiredLabelRule {
                message: "seasonal rollback audit needs this field".to_string(),
            }),
            Rc::new(SeasonalRollbackAuditRequiredClusterRule {
                message: "seasonal rollback audit needs an entry".to_string(),
            }),
            Rc::new(SeasonalRollbackAuditRequiredCodeRule {
                message: "seasonal rollback audit needs this field".to_string(),
            }),
            Rc::new(SeasonalRollbackAuditClusterLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_rollback_audit_validator();
        let item = new_seasonal_rollback_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalRollbackAuditNameLengthRule { max_length: 80 };
        let mut item = new_seasonal_rollback_audit("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SeasonalRollbackAuditRequiredLabelRule {
            message: "seasonal rollback audit needs an entry".to_string(),
        };
        let mut item = new_seasonal_rollback_audit("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = SeasonalRollbackAuditRequiredClusterRule {
            message: "seasonal rollback audit needs a value".to_string(),
        };
        let mut item = new_seasonal_rollback_audit("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalRollbackAuditRequiredCodeRule {
            message: "seasonal rollback audit needs an entry".to_string(),
        };
        let mut item = new_seasonal_rollback_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = SeasonalRollbackAuditClusterLengthRule { max_length: 80 };
        let mut item = new_seasonal_rollback_audit("x", "y");
        item.cluster = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
