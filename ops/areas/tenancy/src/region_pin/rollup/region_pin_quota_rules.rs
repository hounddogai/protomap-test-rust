use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin quota must pass. Check returns its problems, or none.
pub trait RegionPinQuotaRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinQuotaRequiredNameRule {
    pub message: String,
}

impl RegionPinQuotaRule for RegionPinQuotaRequiredNameRule {
    fn check(&self, item: &RegionPinQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinQuotaQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinQuotaRule for RegionPinQuotaQuantityRangeRule {
    fn check(&self, item: &RegionPinQuota) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinQuotaReviewerLengthRule {
    pub max_length: i64,
}

impl RegionPinQuotaRule for RegionPinQuotaReviewerLengthRule {
    fn check(&self, item: &RegionPinQuota) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinQuotaCodeLengthRule {
    pub max_length: i64,
}

impl RegionPinQuotaRule for RegionPinQuotaCodeLengthRule {
    fn check(&self, item: &RegionPinQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinQuotaActiveGuardRule {
    pub limit: i64,
}

impl RegionPinQuotaRule for RegionPinQuotaActiveGuardRule {
    fn check(&self, item: &RegionPinQuota) -> Vec<String> {
        if item.active && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while active",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinQuotaBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionPinQuotaRule for RegionPinQuotaBudgetRangeRule {
    fn check(&self, item: &RegionPinQuota) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin quota.
#[derive(Debug, Clone)]
pub struct RegionPinQuotaValidator {
    pub rules: Vec<Rc<dyn RegionPinQuotaRule>>,
}

impl RegionPinQuotaValidator {
    pub fn validate(&self, item: &RegionPinQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_quota_validator() -> RegionPinQuotaValidator {
    RegionPinQuotaValidator {
        rules: vec![
            Rc::new(RegionPinQuotaRequiredNameRule {
                message: "region pin quota needs this field".to_string(),
            }) as Rc<dyn RegionPinQuotaRule>,
            Rc::new(RegionPinQuotaQuantityRangeRule { min: 120, max: 120 }),
            Rc::new(RegionPinQuotaReviewerLengthRule { max_length: 80 }),
            Rc::new(RegionPinQuotaCodeLengthRule { max_length: 1000 }),
            Rc::new(RegionPinQuotaActiveGuardRule { limit: 3600 }),
            Rc::new(RegionPinQuotaBudgetRangeRule { min: 6.8, max: 4841.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_quota_validator();
        let item = new_region_pin_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionPinQuotaRequiredNameRule {
            message: "region pin quota needs an entry".to_string(),
        };
        let mut item = new_region_pin_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = RegionPinQuotaQuantityRangeRule { min: 64, max: 3600 };
        let mut item = new_region_pin_quota("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = RegionPinQuotaReviewerLengthRule { max_length: 86400 };
        let mut item = new_region_pin_quota("x", "y");
        item.reviewer = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionPinQuotaCodeLengthRule { max_length: 64 };
        let mut item = new_region_pin_quota("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RegionPinQuotaBudgetRangeRule { min: 5.8, max: 3967.9 };
        let mut item = new_region_pin_quota("x", "y");
        item.budget = 3969.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
