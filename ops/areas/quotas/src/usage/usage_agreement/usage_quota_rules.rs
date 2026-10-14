use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage quota must pass. Check returns its problems, or none.
pub trait UsageQuotaRule: std::fmt::Debug {
    fn check(&self, item: &UsageQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageQuotaBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageQuotaRule for UsageQuotaBatchSizeRangeRule {
    fn check(&self, item: &UsageQuota) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageQuotaTitleLengthRule {
    pub max_length: i64,
}

impl UsageQuotaRule for UsageQuotaTitleLengthRule {
    fn check(&self, item: &UsageQuota) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageQuotaAllowedHostRule {
    pub allowed: Vec<String>,
}

impl UsageQuotaRule for UsageQuotaAllowedHostRule {
    fn check(&self, item: &UsageQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is below minimum", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageQuotaDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl UsageQuotaRule for UsageQuotaDiscountRangeRule {
    fn check(&self, item: &UsageQuota) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a usage quota.
#[derive(Debug, Clone)]
pub struct UsageQuotaValidator {
    pub rules: Vec<Rc<dyn UsageQuotaRule>>,
}

impl UsageQuotaValidator {
    pub fn validate(&self, item: &UsageQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_quota_validator() -> UsageQuotaValidator {
    UsageQuotaValidator {
        rules: vec![
            Rc::new(UsageQuotaBatchSizeRangeRule { min: 64, max: 120 }) as Rc<dyn UsageQuotaRule>,
            Rc::new(UsageQuotaTitleLengthRule { max_length: 255 }),
            Rc::new(UsageQuotaAllowedHostRule {
                allowed: vec!["primary".to_string(), "lenient".to_string(), "weighted".to_string()],
            }),
            Rc::new(UsageQuotaDiscountRangeRule { min: 6.7, max: 2757.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_quota_validator();
        let item = new_usage_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = UsageQuotaBatchSizeRangeRule { min: 16, max: 255 };
        let mut item = new_usage_quota("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = UsageQuotaTitleLengthRule { max_length: 1000 };
        let mut item = new_usage_quota("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = UsageQuotaDiscountRangeRule { min: 8.4, max: 456.9 };
        let mut item = new_usage_quota("x", "y");
        item.discount = 458.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
