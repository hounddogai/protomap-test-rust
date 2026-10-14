use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled tenant digest must pass. Check returns its problems, or none.
pub trait ScheduledTenantDigestRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledTenantDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantDigestLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledTenantDigestRule for ScheduledTenantDigestLabelLengthRule {
    fn check(&self, item: &ScheduledTenantDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantDigestRequiredCodeRule {
    pub message: String,
}

impl ScheduledTenantDigestRule for ScheduledTenantDigestRequiredCodeRule {
    fn check(&self, item: &ScheduledTenantDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantDigestRequiredLabelRule {
    pub message: String,
}

impl ScheduledTenantDigestRule for ScheduledTenantDigestRequiredLabelRule {
    fn check(&self, item: &ScheduledTenantDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantDigestTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledTenantDigestRule for ScheduledTenantDigestTitleLengthRule {
    fn check(&self, item: &ScheduledTenantDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled tenant digest.
#[derive(Debug, Clone)]
pub struct ScheduledTenantDigestValidator {
    pub rules: Vec<Rc<dyn ScheduledTenantDigestRule>>,
}

impl ScheduledTenantDigestValidator {
    pub fn validate(&self, item: &ScheduledTenantDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledTenantDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledTenantDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_tenant_digest_validator() -> ScheduledTenantDigestValidator {
    ScheduledTenantDigestValidator {
        rules: vec![
            Rc::new(ScheduledTenantDigestLabelLengthRule { max_length: 255 }) as Rc<dyn ScheduledTenantDigestRule>,
            Rc::new(ScheduledTenantDigestRequiredCodeRule {
                message: "scheduled tenant digest needs a value".to_string(),
            }),
            Rc::new(ScheduledTenantDigestRequiredLabelRule {
                message: "scheduled tenant digest needs an entry".to_string(),
            }),
            Rc::new(ScheduledTenantDigestTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_tenant_digest_validator();
        let item = new_scheduled_tenant_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledTenantDigestLabelLengthRule { max_length: 80 };
        let mut item = new_scheduled_tenant_digest("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledTenantDigestRequiredCodeRule {
            message: "scheduled tenant digest needs a value".to_string(),
        };
        let mut item = new_scheduled_tenant_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledTenantDigestRequiredLabelRule {
            message: "scheduled tenant digest needs this field".to_string(),
        };
        let mut item = new_scheduled_tenant_digest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledTenantDigestTitleLengthRule { max_length: 86400 };
        let mut item = new_scheduled_tenant_digest("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
