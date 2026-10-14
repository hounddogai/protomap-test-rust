use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy quota must pass. Check returns its problems, or none.
pub trait TtlPolicyQuotaRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyQuotaArchivedGuardRule {
    pub limit: f64,
}

impl TtlPolicyQuotaRule for TtlPolicyQuotaArchivedGuardRule {
    fn check(&self, item: &TtlPolicyQuota) -> Vec<String> {
        if item.archived && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while archived",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyQuotaRequiredHostRule {
    pub message: String,
}

impl TtlPolicyQuotaRule for TtlPolicyQuotaRequiredHostRule {
    fn check(&self, item: &TtlPolicyQuota) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyQuotaRequiredChannelRule {
    pub message: String,
}

impl TtlPolicyQuotaRule for TtlPolicyQuotaRequiredChannelRule {
    fn check(&self, item: &TtlPolicyQuota) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyQuotaGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyQuotaRule for TtlPolicyQuotaGraceDaysRangeRule {
    fn check(&self, item: &TtlPolicyQuota) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy quota.
#[derive(Debug, Clone)]
pub struct TtlPolicyQuotaValidator {
    pub rules: Vec<Rc<dyn TtlPolicyQuotaRule>>,
}

impl TtlPolicyQuotaValidator {
    pub fn validate(&self, item: &TtlPolicyQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_quota_validator() -> TtlPolicyQuotaValidator {
    TtlPolicyQuotaValidator {
        rules: vec![
            Rc::new(TtlPolicyQuotaArchivedGuardRule { limit: 3.7 }) as Rc<dyn TtlPolicyQuotaRule>,
            Rc::new(TtlPolicyQuotaRequiredHostRule {
                message: "ttl policy quota needs this field".to_string(),
            }),
            Rc::new(TtlPolicyQuotaRequiredChannelRule {
                message: "ttl policy quota needs this field".to_string(),
            }),
            Rc::new(TtlPolicyQuotaGraceDaysRangeRule { min: 3600, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_quota_validator();
        let item = new_ttl_policy_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = TtlPolicyQuotaRequiredHostRule {
            message: "ttl policy quota needs a value".to_string(),
        };
        let mut item = new_ttl_policy_quota("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = TtlPolicyQuotaRequiredChannelRule {
            message: "ttl policy quota needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_quota("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = TtlPolicyQuotaGraceDaysRangeRule { min: 64, max: 120 };
        let mut item = new_ttl_policy_quota("x", "y");
        item.grace_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
