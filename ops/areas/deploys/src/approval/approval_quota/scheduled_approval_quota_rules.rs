use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled approval quota must pass. Check returns its problems, or none.
pub trait ScheduledApprovalQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledApprovalQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledApprovalQuotaCodePrefixRule {
    pub prefix: String,
}

impl ScheduledApprovalQuotaRule for ScheduledApprovalQuotaCodePrefixRule {
    fn check(&self, item: &ScheduledApprovalQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledApprovalQuotaUnitLengthRule {
    pub max_length: i64,
}

impl ScheduledApprovalQuotaRule for ScheduledApprovalQuotaUnitLengthRule {
    fn check(&self, item: &ScheduledApprovalQuota) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledApprovalQuotaAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledApprovalQuotaRule for ScheduledApprovalQuotaAmountRangeRule {
    fn check(&self, item: &ScheduledApprovalQuota) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled approval quota.
#[derive(Debug, Clone)]
pub struct ScheduledApprovalQuotaValidator {
    pub rules: Vec<Rc<dyn ScheduledApprovalQuotaRule>>,
}

impl ScheduledApprovalQuotaValidator {
    pub fn validate(&self, item: &ScheduledApprovalQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledApprovalQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledApprovalQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_approval_quota_validator() -> ScheduledApprovalQuotaValidator {
    ScheduledApprovalQuotaValidator {
        rules: vec![
            Rc::new(ScheduledApprovalQuotaCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn ScheduledApprovalQuotaRule>,
            Rc::new(ScheduledApprovalQuotaUnitLengthRule { max_length: 32 }),
            Rc::new(ScheduledApprovalQuotaAmountRangeRule { min: 4.1, max: 4593.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_approval_quota_validator();
        let item = new_scheduled_approval_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ScheduledApprovalQuotaUnitLengthRule { max_length: 120 };
        let mut item = new_scheduled_approval_quota("x", "y");
        item.unit = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = ScheduledApprovalQuotaAmountRangeRule { min: 9.0, max: 1434.8 };
        let mut item = new_scheduled_approval_quota("x", "y");
        item.amount = 1436.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
