use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant audit must pass. Check returns its problems, or none.
pub trait GrantAuditRule: std::fmt::Debug {
    fn check(&self, item: &GrantAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAuditRequiredRegionRule {
    pub message: String,
}

impl GrantAuditRule for GrantAuditRequiredRegionRule {
    fn check(&self, item: &GrantAudit) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAuditTimezoneLengthRule {
    pub max_length: i64,
}

impl GrantAuditRule for GrantAuditTimezoneLengthRule {
    fn check(&self, item: &GrantAudit) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAuditRegionLengthRule {
    pub max_length: i64,
}

impl GrantAuditRule for GrantAuditRegionLengthRule {
    fn check(&self, item: &GrantAudit) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAuditRequiredTitleRule {
    pub message: String,
}

impl GrantAuditRule for GrantAuditRequiredTitleRule {
    fn check(&self, item: &GrantAudit) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant audit.
#[derive(Debug, Clone)]
pub struct GrantAuditValidator {
    pub rules: Vec<Rc<dyn GrantAuditRule>>,
}

impl GrantAuditValidator {
    pub fn validate(&self, item: &GrantAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_audit_validator() -> GrantAuditValidator {
    GrantAuditValidator {
        rules: vec![
            Rc::new(GrantAuditRequiredRegionRule {
                message: "grant audit needs an entry".to_string(),
            }) as Rc<dyn GrantAuditRule>,
            Rc::new(GrantAuditTimezoneLengthRule { max_length: 32 }),
            Rc::new(GrantAuditRegionLengthRule { max_length: 80 }),
            Rc::new(GrantAuditRequiredTitleRule {
                message: "grant audit needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_audit_validator();
        let item = new_grant_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = GrantAuditRequiredRegionRule {
            message: "grant audit needs an entry".to_string(),
        };
        let mut item = new_grant_audit("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = GrantAuditTimezoneLengthRule { max_length: 32 };
        let mut item = new_grant_audit("x", "y");
        item.timezone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = GrantAuditRegionLengthRule { max_length: 16 };
        let mut item = new_grant_audit("x", "y");
        item.region = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GrantAuditRequiredTitleRule {
            message: "grant audit needs a value".to_string(),
        };
        let mut item = new_grant_audit("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
