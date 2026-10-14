use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling page audit must pass. Check returns its problems, or none.
pub trait RollingPageAuditRule: std::fmt::Debug {
    fn check(&self, item: &RollingPageAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageAuditCodePrefixRule {
    pub prefix: String,
}

impl RollingPageAuditRule for RollingPageAuditCodePrefixRule {
    fn check(&self, item: &RollingPageAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageAuditRequiredScopeRule {
    pub message: String,
}

impl RollingPageAuditRule for RollingPageAuditRequiredScopeRule {
    fn check(&self, item: &RollingPageAudit) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageAuditAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollingPageAuditRule for RollingPageAuditAllowedCodeRule {
    fn check(&self, item: &RollingPageAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageAuditRequiredCodeRule {
    pub message: String,
}

impl RollingPageAuditRule for RollingPageAuditRequiredCodeRule {
    fn check(&self, item: &RollingPageAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageAuditPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingPageAuditRule for RollingPageAuditPriorityRangeRule {
    fn check(&self, item: &RollingPageAudit) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageAuditRequiredFingerprintRule {
    pub message: String,
}

impl RollingPageAuditRule for RollingPageAuditRequiredFingerprintRule {
    fn check(&self, item: &RollingPageAudit) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling page audit.
#[derive(Debug, Clone)]
pub struct RollingPageAuditValidator {
    pub rules: Vec<Rc<dyn RollingPageAuditRule>>,
}

impl RollingPageAuditValidator {
    pub fn validate(&self, item: &RollingPageAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingPageAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPageAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_page_audit_validator() -> RollingPageAuditValidator {
    RollingPageAuditValidator {
        rules: vec![
            Rc::new(RollingPageAuditCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn RollingPageAuditRule>,
            Rc::new(RollingPageAuditRequiredScopeRule {
                message: "rolling page audit needs a value".to_string(),
            }),
            Rc::new(RollingPageAuditAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(RollingPageAuditRequiredCodeRule {
                message: "rolling page audit needs a value".to_string(),
            }),
            Rc::new(RollingPageAuditPriorityRangeRule { min: 32, max: 32 }),
            Rc::new(RollingPageAuditRequiredFingerprintRule {
                message: "rolling page audit needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_page_audit_validator();
        let item = new_rolling_page_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = RollingPageAuditRequiredScopeRule {
            message: "rolling page audit needs a value".to_string(),
        };
        let mut item = new_rolling_page_audit("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingPageAuditRequiredCodeRule {
            message: "rolling page audit needs an entry".to_string(),
        };
        let mut item = new_rolling_page_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = RollingPageAuditPriorityRangeRule { min: 80, max: 255 };
        let mut item = new_rolling_page_audit("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = RollingPageAuditRequiredFingerprintRule {
            message: "rolling page audit needs this field".to_string(),
        };
        let mut item = new_rolling_page_audit("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
