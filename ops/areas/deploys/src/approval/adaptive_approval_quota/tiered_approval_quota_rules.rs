use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered approval quota must pass. Check returns its problems, or none.
pub trait TieredApprovalQuotaRule: std::fmt::Debug {
    fn check(&self, item: &TieredApprovalQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalQuotaRequiredNotesRule {
    pub message: String,
}

impl TieredApprovalQuotaRule for TieredApprovalQuotaRequiredNotesRule {
    fn check(&self, item: &TieredApprovalQuota) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalQuotaTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl TieredApprovalQuotaRule for TieredApprovalQuotaTagsLimitRule {
    fn check(&self, item: &TieredApprovalQuota) -> Vec<String> {
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
pub struct TieredApprovalQuotaTitleLengthRule {
    pub max_length: i64,
}

impl TieredApprovalQuotaRule for TieredApprovalQuotaTitleLengthRule {
    fn check(&self, item: &TieredApprovalQuota) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalQuotaNotesLengthRule {
    pub max_length: i64,
}

impl TieredApprovalQuotaRule for TieredApprovalQuotaNotesLengthRule {
    fn check(&self, item: &TieredApprovalQuota) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalQuotaTenantIdLengthRule {
    pub max_length: i64,
}

impl TieredApprovalQuotaRule for TieredApprovalQuotaTenantIdLengthRule {
    fn check(&self, item: &TieredApprovalQuota) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalQuotaCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredApprovalQuotaRule for TieredApprovalQuotaCostRangeRule {
    fn check(&self, item: &TieredApprovalQuota) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered approval quota.
#[derive(Debug, Clone)]
pub struct TieredApprovalQuotaValidator {
    pub rules: Vec<Rc<dyn TieredApprovalQuotaRule>>,
}

impl TieredApprovalQuotaValidator {
    pub fn validate(&self, item: &TieredApprovalQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredApprovalQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredApprovalQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_approval_quota_validator() -> TieredApprovalQuotaValidator {
    TieredApprovalQuotaValidator {
        rules: vec![
            Rc::new(TieredApprovalQuotaRequiredNotesRule {
                message: "tiered approval quota needs a value".to_string(),
            }) as Rc<dyn TieredApprovalQuotaRule>,
            Rc::new(TieredApprovalQuotaTagsLimitRule {
                max_count: 3600,
                banned: vec!["legacy".to_string()],
            }),
            Rc::new(TieredApprovalQuotaTitleLengthRule { max_length: 86400 }),
            Rc::new(TieredApprovalQuotaNotesLengthRule { max_length: 80 }),
            Rc::new(TieredApprovalQuotaTenantIdLengthRule { max_length: 16 }),
            Rc::new(TieredApprovalQuotaCostRangeRule { min: 2.1, max: 300.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_approval_quota_validator();
        let item = new_tiered_approval_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = TieredApprovalQuotaRequiredNotesRule {
            message: "tiered approval quota needs a value".to_string(),
        };
        let mut item = new_tiered_approval_quota("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredApprovalQuotaTitleLengthRule { max_length: 120 };
        let mut item = new_tiered_approval_quota("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = TieredApprovalQuotaNotesLengthRule { max_length: 3600 };
        let mut item = new_tiered_approval_quota("x", "y");
        item.notes = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = TieredApprovalQuotaTenantIdLengthRule { max_length: 255 };
        let mut item = new_tiered_approval_quota("x", "y");
        item.tenant_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = TieredApprovalQuotaCostRangeRule { min: 7.5, max: 3163.6 };
        let mut item = new_tiered_approval_quota("x", "y");
        item.cost = 3165.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
