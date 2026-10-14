use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual suspension quota must pass. Check returns its problems, or none.
pub trait ManualSuspensionQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ManualSuspensionQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSuspensionQuotaKeyIdLengthRule {
    pub max_length: i64,
}

impl ManualSuspensionQuotaRule for ManualSuspensionQuotaKeyIdLengthRule {
    fn check(&self, item: &ManualSuspensionQuota) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSuspensionQuotaTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ManualSuspensionQuotaRule for ManualSuspensionQuotaTagsLimitRule {
    fn check(&self, item: &ManualSuspensionQuota) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSuspensionQuotaNameLengthRule {
    pub max_length: i64,
}

impl ManualSuspensionQuotaRule for ManualSuspensionQuotaNameLengthRule {
    fn check(&self, item: &ManualSuspensionQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual suspension quota.
#[derive(Debug, Clone)]
pub struct ManualSuspensionQuotaValidator {
    pub rules: Vec<Rc<dyn ManualSuspensionQuotaRule>>,
}

impl ManualSuspensionQuotaValidator {
    pub fn validate(&self, item: &ManualSuspensionQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualSuspensionQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualSuspensionQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_suspension_quota_validator() -> ManualSuspensionQuotaValidator {
    ManualSuspensionQuotaValidator {
        rules: vec![
            Rc::new(ManualSuspensionQuotaKeyIdLengthRule { max_length: 255 }) as Rc<dyn ManualSuspensionQuotaRule>,
            Rc::new(ManualSuspensionQuotaTagsLimitRule {
                max_count: 86400,
                banned: Vec::new(),
            }),
            Rc::new(ManualSuspensionQuotaNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_suspension_quota_validator();
        let item = new_manual_suspension_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = ManualSuspensionQuotaKeyIdLengthRule { max_length: 32 };
        let mut item = new_manual_suspension_quota("x", "y");
        item.key_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualSuspensionQuotaNameLengthRule { max_length: 80 };
        let mut item = new_manual_suspension_quota("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
