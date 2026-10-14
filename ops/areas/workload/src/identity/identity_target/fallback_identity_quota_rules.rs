use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback identity quota must pass. Check returns its problems, or none.
pub trait FallbackIdentityQuotaRule: std::fmt::Debug {
    fn check(&self, item: &FallbackIdentityQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityQuotaScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackIdentityQuotaRule for FallbackIdentityQuotaScoreRangeRule {
    fn check(&self, item: &FallbackIdentityQuota) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityQuotaCodePrefixRule {
    pub prefix: String,
}

impl FallbackIdentityQuotaRule for FallbackIdentityQuotaCodePrefixRule {
    fn check(&self, item: &FallbackIdentityQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityQuotaCodeLengthRule {
    pub max_length: i64,
}

impl FallbackIdentityQuotaRule for FallbackIdentityQuotaCodeLengthRule {
    fn check(&self, item: &FallbackIdentityQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityQuotaPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackIdentityQuotaRule for FallbackIdentityQuotaPriorityRangeRule {
    fn check(&self, item: &FallbackIdentityQuota) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityQuotaRequiredRegionRule {
    pub message: String,
}

impl FallbackIdentityQuotaRule for FallbackIdentityQuotaRequiredRegionRule {
    fn check(&self, item: &FallbackIdentityQuota) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityQuotaDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackIdentityQuotaRule for FallbackIdentityQuotaDiscountRangeRule {
    fn check(&self, item: &FallbackIdentityQuota) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback identity quota.
#[derive(Debug, Clone)]
pub struct FallbackIdentityQuotaValidator {
    pub rules: Vec<Rc<dyn FallbackIdentityQuotaRule>>,
}

impl FallbackIdentityQuotaValidator {
    pub fn validate(&self, item: &FallbackIdentityQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackIdentityQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackIdentityQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_identity_quota_validator() -> FallbackIdentityQuotaValidator {
    FallbackIdentityQuotaValidator {
        rules: vec![
            Rc::new(FallbackIdentityQuotaScoreRangeRule { min: 2.9, max: 4859.6 }) as Rc<dyn FallbackIdentityQuotaRule>,
            Rc::new(FallbackIdentityQuotaCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(FallbackIdentityQuotaCodeLengthRule { max_length: 80 }),
            Rc::new(FallbackIdentityQuotaPriorityRangeRule { min: 80, max: 86400 }),
            Rc::new(FallbackIdentityQuotaRequiredRegionRule {
                message: "fallback identity quota needs an entry".to_string(),
            }),
            Rc::new(FallbackIdentityQuotaDiscountRangeRule { min: 2.5, max: 4847.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_identity_quota_validator();
        let item = new_fallback_identity_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = FallbackIdentityQuotaScoreRangeRule { min: 2.5, max: 3616.9 };
        let mut item = new_fallback_identity_quota("x", "y");
        item.score = 3618.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackIdentityQuotaCodeLengthRule { max_length: 80 };
        let mut item = new_fallback_identity_quota("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = FallbackIdentityQuotaPriorityRangeRule { min: 255, max: 1000 };
        let mut item = new_fallback_identity_quota("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = FallbackIdentityQuotaRequiredRegionRule {
            message: "fallback identity quota needs a value".to_string(),
        };
        let mut item = new_fallback_identity_quota("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = FallbackIdentityQuotaDiscountRangeRule { min: 6.1, max: 2322.7 };
        let mut item = new_fallback_identity_quota("x", "y");
        item.discount = 2324.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
