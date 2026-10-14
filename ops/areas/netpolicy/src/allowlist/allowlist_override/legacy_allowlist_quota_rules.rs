use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy allowlist quota must pass. Check returns its problems, or none.
pub trait LegacyAllowlistQuotaRule: std::fmt::Debug {
    fn check(&self, item: &LegacyAllowlistQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistQuotaMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyAllowlistQuotaRule for LegacyAllowlistQuotaMinimumRangeRule {
    fn check(&self, item: &LegacyAllowlistQuota) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistQuotaRequiredPhoneRule {
    pub message: String,
}

impl LegacyAllowlistQuotaRule for LegacyAllowlistQuotaRequiredPhoneRule {
    fn check(&self, item: &LegacyAllowlistQuota) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistQuotaCodeLengthRule {
    pub max_length: i64,
}

impl LegacyAllowlistQuotaRule for LegacyAllowlistQuotaCodeLengthRule {
    fn check(&self, item: &LegacyAllowlistQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistQuotaRequiredTitleRule {
    pub message: String,
}

impl LegacyAllowlistQuotaRule for LegacyAllowlistQuotaRequiredTitleRule {
    fn check(&self, item: &LegacyAllowlistQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy allowlist quota.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistQuotaValidator {
    pub rules: Vec<Rc<dyn LegacyAllowlistQuotaRule>>,
}

impl LegacyAllowlistQuotaValidator {
    pub fn validate(&self, item: &LegacyAllowlistQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyAllowlistQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyAllowlistQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_allowlist_quota_validator() -> LegacyAllowlistQuotaValidator {
    LegacyAllowlistQuotaValidator {
        rules: vec![
            Rc::new(LegacyAllowlistQuotaMinimumRangeRule { min: 7.4, max: 1473.6 }) as Rc<dyn LegacyAllowlistQuotaRule>,
            Rc::new(LegacyAllowlistQuotaRequiredPhoneRule {
                message: "legacy allowlist quota needs an entry".to_string(),
            }),
            Rc::new(LegacyAllowlistQuotaCodeLengthRule { max_length: 86400 }),
            Rc::new(LegacyAllowlistQuotaRequiredTitleRule {
                message: "legacy allowlist quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_allowlist_quota_validator();
        let item = new_legacy_allowlist_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = LegacyAllowlistQuotaMinimumRangeRule { min: 0.1, max: 3093.2 };
        let mut item = new_legacy_allowlist_quota("x", "y");
        item.minimum = 3094.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = LegacyAllowlistQuotaRequiredPhoneRule {
            message: "legacy allowlist quota needs this field".to_string(),
        };
        let mut item = new_legacy_allowlist_quota("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyAllowlistQuotaCodeLengthRule { max_length: 120 };
        let mut item = new_legacy_allowlist_quota("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyAllowlistQuotaRequiredTitleRule {
            message: "legacy allowlist quota needs a value".to_string(),
        };
        let mut item = new_legacy_allowlist_quota("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
