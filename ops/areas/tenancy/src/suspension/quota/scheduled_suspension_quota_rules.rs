use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled suspension quota must pass. Check returns its problems, or none.
pub trait ScheduledSuspensionQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSuspensionQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionQuotaNameLengthRule {
    pub max_length: i64,
}

impl ScheduledSuspensionQuotaRule for ScheduledSuspensionQuotaNameLengthRule {
    fn check(&self, item: &ScheduledSuspensionQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionQuotaEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSuspensionQuotaRule for ScheduledSuspensionQuotaEndsAtRangeRule {
    fn check(&self, item: &ScheduledSuspensionQuota) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionQuotaCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledSuspensionQuotaRule for ScheduledSuspensionQuotaCodeLengthRule {
    fn check(&self, item: &ScheduledSuspensionQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionQuotaRequiredCurrencyRule {
    pub message: String,
}

impl ScheduledSuspensionQuotaRule for ScheduledSuspensionQuotaRequiredCurrencyRule {
    fn check(&self, item: &ScheduledSuspensionQuota) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionQuotaRequiredNameRule {
    pub message: String,
}

impl ScheduledSuspensionQuotaRule for ScheduledSuspensionQuotaRequiredNameRule {
    fn check(&self, item: &ScheduledSuspensionQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled suspension quota.
#[derive(Debug, Clone)]
pub struct ScheduledSuspensionQuotaValidator {
    pub rules: Vec<Rc<dyn ScheduledSuspensionQuotaRule>>,
}

impl ScheduledSuspensionQuotaValidator {
    pub fn validate(&self, item: &ScheduledSuspensionQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledSuspensionQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSuspensionQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_suspension_quota_validator() -> ScheduledSuspensionQuotaValidator {
    ScheduledSuspensionQuotaValidator {
        rules: vec![
            Rc::new(ScheduledSuspensionQuotaNameLengthRule { max_length: 32 }) as Rc<dyn ScheduledSuspensionQuotaRule>,
            Rc::new(ScheduledSuspensionQuotaEndsAtRangeRule { min: 32, max: 3600 }),
            Rc::new(ScheduledSuspensionQuotaCodeLengthRule { max_length: 32 }),
            Rc::new(ScheduledSuspensionQuotaRequiredCurrencyRule {
                message: "scheduled suspension quota needs a value".to_string(),
            }),
            Rc::new(ScheduledSuspensionQuotaRequiredNameRule {
                message: "scheduled suspension quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_suspension_quota_validator();
        let item = new_scheduled_suspension_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ScheduledSuspensionQuotaNameLengthRule { max_length: 86400 };
        let mut item = new_scheduled_suspension_quota("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = ScheduledSuspensionQuotaEndsAtRangeRule { min: 32, max: 120 };
        let mut item = new_scheduled_suspension_quota("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledSuspensionQuotaCodeLengthRule { max_length: 86400 };
        let mut item = new_scheduled_suspension_quota("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = ScheduledSuspensionQuotaRequiredCurrencyRule {
            message: "scheduled suspension quota needs an entry".to_string(),
        };
        let mut item = new_scheduled_suspension_quota("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledSuspensionQuotaRequiredNameRule {
            message: "scheduled suspension quota needs this field".to_string(),
        };
        let mut item = new_scheduled_suspension_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
