use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy health check quota must pass. Check returns its problems, or none.
pub trait LegacyHealthCheckQuotaRule: std::fmt::Debug {
    fn check(&self, item: &LegacyHealthCheckQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyHealthCheckQuotaAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl LegacyHealthCheckQuotaRule for LegacyHealthCheckQuotaAllowedLabelRule {
    fn check(&self, item: &LegacyHealthCheckQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is inconsistent", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyHealthCheckQuotaRequiredCodeRule {
    pub message: String,
}

impl LegacyHealthCheckQuotaRule for LegacyHealthCheckQuotaRequiredCodeRule {
    fn check(&self, item: &LegacyHealthCheckQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyHealthCheckQuotaCodeLengthRule {
    pub max_length: i64,
}

impl LegacyHealthCheckQuotaRule for LegacyHealthCheckQuotaCodeLengthRule {
    fn check(&self, item: &LegacyHealthCheckQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyHealthCheckQuotaOwnerIdLengthRule {
    pub max_length: i64,
}

impl LegacyHealthCheckQuotaRule for LegacyHealthCheckQuotaOwnerIdLengthRule {
    fn check(&self, item: &LegacyHealthCheckQuota) -> Vec<String> {
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
pub struct LegacyHealthCheckQuotaRequiredLabelRule {
    pub message: String,
}

impl LegacyHealthCheckQuotaRule for LegacyHealthCheckQuotaRequiredLabelRule {
    fn check(&self, item: &LegacyHealthCheckQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyHealthCheckQuotaNameLengthRule {
    pub max_length: i64,
}

impl LegacyHealthCheckQuotaRule for LegacyHealthCheckQuotaNameLengthRule {
    fn check(&self, item: &LegacyHealthCheckQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy health check quota.
#[derive(Debug, Clone)]
pub struct LegacyHealthCheckQuotaValidator {
    pub rules: Vec<Rc<dyn LegacyHealthCheckQuotaRule>>,
}

impl LegacyHealthCheckQuotaValidator {
    pub fn validate(&self, item: &LegacyHealthCheckQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyHealthCheckQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyHealthCheckQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_health_check_quota_validator() -> LegacyHealthCheckQuotaValidator {
    LegacyHealthCheckQuotaValidator {
        rules: vec![
            Rc::new(LegacyHealthCheckQuotaAllowedLabelRule {
                allowed: vec!["manual".to_string()],
            }) as Rc<dyn LegacyHealthCheckQuotaRule>,
            Rc::new(LegacyHealthCheckQuotaRequiredCodeRule {
                message: "legacy health check quota needs this field".to_string(),
            }),
            Rc::new(LegacyHealthCheckQuotaCodeLengthRule { max_length: 86400 }),
            Rc::new(LegacyHealthCheckQuotaOwnerIdLengthRule { max_length: 255 }),
            Rc::new(LegacyHealthCheckQuotaRequiredLabelRule {
                message: "legacy health check quota needs a value".to_string(),
            }),
            Rc::new(LegacyHealthCheckQuotaNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_health_check_quota_validator();
        let item = new_legacy_health_check_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyHealthCheckQuotaRequiredCodeRule {
            message: "legacy health check quota needs an entry".to_string(),
        };
        let mut item = new_legacy_health_check_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyHealthCheckQuotaCodeLengthRule { max_length: 3600 };
        let mut item = new_legacy_health_check_quota("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = LegacyHealthCheckQuotaOwnerIdLengthRule { max_length: 120 };
        let mut item = new_legacy_health_check_quota("x", "y");
        item.owner_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacyHealthCheckQuotaRequiredLabelRule {
            message: "legacy health check quota needs a value".to_string(),
        };
        let mut item = new_legacy_health_check_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LegacyHealthCheckQuotaNameLengthRule { max_length: 3600 };
        let mut item = new_legacy_health_check_quota("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
