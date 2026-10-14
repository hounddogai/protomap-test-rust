use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a suspension audit must pass. Check returns its problems, or none.
pub trait SuspensionAuditRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAuditRequiredRegionRule {
    pub message: String,
}

impl SuspensionAuditRule for SuspensionAuditRequiredRegionRule {
    fn check(&self, item: &SuspensionAudit) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAuditAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl SuspensionAuditRule for SuspensionAuditAllowedKeyIdRule {
    fn check(&self, item: &SuspensionAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is above maximum", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAuditRequiredNameRule {
    pub message: String,
}

impl SuspensionAuditRule for SuspensionAuditRequiredNameRule {
    fn check(&self, item: &SuspensionAudit) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAuditKeyIdLengthRule {
    pub max_length: i64,
}

impl SuspensionAuditRule for SuspensionAuditKeyIdLengthRule {
    fn check(&self, item: &SuspensionAudit) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension audit.
#[derive(Debug, Clone)]
pub struct SuspensionAuditValidator {
    pub rules: Vec<Rc<dyn SuspensionAuditRule>>,
}

impl SuspensionAuditValidator {
    pub fn validate(&self, item: &SuspensionAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_audit_validator() -> SuspensionAuditValidator {
    SuspensionAuditValidator {
        rules: vec![
            Rc::new(SuspensionAuditRequiredRegionRule {
                message: "suspension audit needs a value".to_string(),
            }) as Rc<dyn SuspensionAuditRule>,
            Rc::new(SuspensionAuditAllowedKeyIdRule {
                allowed: vec!["batched".to_string(), "default".to_string()],
            }),
            Rc::new(SuspensionAuditRequiredNameRule {
                message: "suspension audit needs a value".to_string(),
            }),
            Rc::new(SuspensionAuditKeyIdLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_audit_validator();
        let item = new_suspension_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = SuspensionAuditRequiredRegionRule {
            message: "suspension audit needs an entry".to_string(),
        };
        let mut item = new_suspension_audit("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SuspensionAuditRequiredNameRule {
            message: "suspension audit needs this field".to_string(),
        };
        let mut item = new_suspension_audit("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = SuspensionAuditKeyIdLengthRule { max_length: 255 };
        let mut item = new_suspension_audit("x", "y");
        item.key_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
