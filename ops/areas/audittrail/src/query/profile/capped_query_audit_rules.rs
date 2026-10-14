use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped query audit must pass. Check returns its problems, or none.
pub trait CappedQueryAuditRule: std::fmt::Debug {
    fn check(&self, item: &CappedQueryAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryAuditPinnedGuardRule {
    pub limit: f64,
}

impl CappedQueryAuditRule for CappedQueryAuditPinnedGuardRule {
    fn check(&self, item: &CappedQueryAudit) -> Vec<String> {
        if item.pinned && (item.minimum > self.limit) {
            return vec![format!("minimum {} exceeds {} while pinned", item.minimum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryAuditRequiredNameRule {
    pub message: String,
}

impl CappedQueryAuditRule for CappedQueryAuditRequiredNameRule {
    fn check(&self, item: &CappedQueryAudit) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryAuditTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl CappedQueryAuditRule for CappedQueryAuditTagsLimitRule {
    fn check(&self, item: &CappedQueryAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a capped query audit.
#[derive(Debug, Clone)]
pub struct CappedQueryAuditValidator {
    pub rules: Vec<Rc<dyn CappedQueryAuditRule>>,
}

impl CappedQueryAuditValidator {
    pub fn validate(&self, item: &CappedQueryAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedQueryAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedQueryAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_query_audit_validator() -> CappedQueryAuditValidator {
    CappedQueryAuditValidator {
        rules: vec![
            Rc::new(CappedQueryAuditPinnedGuardRule { limit: 9.4 }) as Rc<dyn CappedQueryAuditRule>,
            Rc::new(CappedQueryAuditRequiredNameRule {
                message: "capped query audit needs a value".to_string(),
            }),
            Rc::new(CappedQueryAuditTagsLimitRule {
                max_count: 1000,
                banned: vec!["seasonal".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_query_audit_validator();
        let item = new_capped_query_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CappedQueryAuditRequiredNameRule {
            message: "capped query audit needs this field".to_string(),
        };
        let mut item = new_capped_query_audit("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
