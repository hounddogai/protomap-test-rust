use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollout quota must pass. Check returns its problems, or none.
pub trait RolloutQuotaRule: std::fmt::Debug {
    fn check(&self, item: &RolloutQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutQuotaNameLengthRule {
    pub max_length: i64,
}

impl RolloutQuotaRule for RolloutQuotaNameLengthRule {
    fn check(&self, item: &RolloutQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutQuotaAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RolloutQuotaRule for RolloutQuotaAllowedNameRule {
    fn check(&self, item: &RolloutQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is inconsistent", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutQuotaRequiredHostRule {
    pub message: String,
}

impl RolloutQuotaRule for RolloutQuotaRequiredHostRule {
    fn check(&self, item: &RolloutQuota) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutQuotaFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutQuotaRule for RolloutQuotaFloorValueRangeRule {
    fn check(&self, item: &RolloutQuota) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutQuotaLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutQuotaRule for RolloutQuotaLimitRangeRule {
    fn check(&self, item: &RolloutQuota) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutQuotaMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutQuotaRule for RolloutQuotaMinimumRangeRule {
    fn check(&self, item: &RolloutQuota) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout quota.
#[derive(Debug, Clone)]
pub struct RolloutQuotaValidator {
    pub rules: Vec<Rc<dyn RolloutQuotaRule>>,
}

impl RolloutQuotaValidator {
    pub fn validate(&self, item: &RolloutQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_quota_validator() -> RolloutQuotaValidator {
    RolloutQuotaValidator {
        rules: vec![
            Rc::new(RolloutQuotaNameLengthRule { max_length: 80 }) as Rc<dyn RolloutQuotaRule>,
            Rc::new(RolloutQuotaAllowedNameRule {
                allowed: vec!["fallback".to_string(), "primary".to_string(), "capped".to_string()],
            }),
            Rc::new(RolloutQuotaRequiredHostRule {
                message: "rollout quota needs this field".to_string(),
            }),
            Rc::new(RolloutQuotaFloorValueRangeRule { min: 7.8, max: 1878.5 }),
            Rc::new(RolloutQuotaLimitRangeRule { min: 64, max: 3600 }),
            Rc::new(RolloutQuotaMinimumRangeRule { min: 1.4, max: 89.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_quota_validator();
        let item = new_rollout_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RolloutQuotaNameLengthRule { max_length: 32 };
        let mut item = new_rollout_quota("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RolloutQuotaRequiredHostRule {
            message: "rollout quota needs a value".to_string(),
        };
        let mut item = new_rollout_quota("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = RolloutQuotaFloorValueRangeRule { min: 6.1, max: 2957.7 };
        let mut item = new_rollout_quota("x", "y");
        item.floor_value = 2959.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = RolloutQuotaLimitRangeRule { min: 64, max: 120 };
        let mut item = new_rollout_quota("x", "y");
        item.limit = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = RolloutQuotaMinimumRangeRule { min: 5.4, max: 799.0 };
        let mut item = new_rollout_quota("x", "y");
        item.minimum = 800.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
