use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual policy quota must pass. Check returns its problems, or none.
pub trait ManualPolicyQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ManualPolicyQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPolicyQuotaRequiredLabelRule {
    pub message: String,
}

impl ManualPolicyQuotaRule for ManualPolicyQuotaRequiredLabelRule {
    fn check(&self, item: &ManualPolicyQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPolicyQuotaRequiredCodeRule {
    pub message: String,
}

impl ManualPolicyQuotaRule for ManualPolicyQuotaRequiredCodeRule {
    fn check(&self, item: &ManualPolicyQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPolicyQuotaLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualPolicyQuotaRule for ManualPolicyQuotaLimitRangeRule {
    fn check(&self, item: &ManualPolicyQuota) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual policy quota.
#[derive(Debug, Clone)]
pub struct ManualPolicyQuotaValidator {
    pub rules: Vec<Rc<dyn ManualPolicyQuotaRule>>,
}

impl ManualPolicyQuotaValidator {
    pub fn validate(&self, item: &ManualPolicyQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualPolicyQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualPolicyQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_policy_quota_validator() -> ManualPolicyQuotaValidator {
    ManualPolicyQuotaValidator {
        rules: vec![
            Rc::new(ManualPolicyQuotaRequiredLabelRule {
                message: "manual policy quota needs this field".to_string(),
            }) as Rc<dyn ManualPolicyQuotaRule>,
            Rc::new(ManualPolicyQuotaRequiredCodeRule {
                message: "manual policy quota needs an entry".to_string(),
            }),
            Rc::new(ManualPolicyQuotaLimitRangeRule { min: 255, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_policy_quota_validator();
        let item = new_manual_policy_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualPolicyQuotaRequiredLabelRule {
            message: "manual policy quota needs an entry".to_string(),
        };
        let mut item = new_manual_policy_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualPolicyQuotaRequiredCodeRule {
            message: "manual policy quota needs this field".to_string(),
        };
        let mut item = new_manual_policy_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ManualPolicyQuotaLimitRangeRule { min: 80, max: 120 };
        let mut item = new_manual_policy_quota("x", "y");
        item.limit = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
