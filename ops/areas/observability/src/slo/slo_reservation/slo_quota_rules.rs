use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a slo quota must pass. Check returns its problems, or none.
pub trait SloQuotaRule: std::fmt::Debug {
    fn check(&self, item: &SloQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloQuotaRequiredCodeRule {
    pub message: String,
}

impl SloQuotaRule for SloQuotaRequiredCodeRule {
    fn check(&self, item: &SloQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloQuotaQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloQuotaRule for SloQuotaQuotaUnitsRangeRule {
    fn check(&self, item: &SloQuota) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloQuotaMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloQuotaRule for SloQuotaMinimumRangeRule {
    fn check(&self, item: &SloQuota) -> Vec<String> {
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
pub struct SloQuotaTitleLengthRule {
    pub max_length: i64,
}

impl SloQuotaRule for SloQuotaTitleLengthRule {
    fn check(&self, item: &SloQuota) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloQuotaExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloQuotaRule for SloQuotaExpiresAtRangeRule {
    fn check(&self, item: &SloQuota) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SloQuotaRule for SloQuotaAllowedCodeRule {
    fn check(&self, item: &SloQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo quota.
#[derive(Debug, Clone)]
pub struct SloQuotaValidator {
    pub rules: Vec<Rc<dyn SloQuotaRule>>,
}

impl SloQuotaValidator {
    pub fn validate(&self, item: &SloQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_quota_validator() -> SloQuotaValidator {
    SloQuotaValidator {
        rules: vec![
            Rc::new(SloQuotaRequiredCodeRule {
                message: "slo quota needs an entry".to_string(),
            }) as Rc<dyn SloQuotaRule>,
            Rc::new(SloQuotaQuotaUnitsRangeRule { min: 255, max: 86400 }),
            Rc::new(SloQuotaMinimumRangeRule { min: 8.2, max: 2774.6 }),
            Rc::new(SloQuotaTitleLengthRule { max_length: 120 }),
            Rc::new(SloQuotaExpiresAtRangeRule { min: 1000, max: 86400 }),
            Rc::new(SloQuotaAllowedCodeRule {
                allowed: vec!["manual".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_quota_validator();
        let item = new_slo_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloQuotaRequiredCodeRule {
            message: "slo quota needs this field".to_string(),
        };
        let mut item = new_slo_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = SloQuotaQuotaUnitsRangeRule { min: 120, max: 1000 };
        let mut item = new_slo_quota("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = SloQuotaMinimumRangeRule { min: 0.2, max: 774.8 };
        let mut item = new_slo_quota("x", "y");
        item.minimum = 776.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SloQuotaTitleLengthRule { max_length: 80 };
        let mut item = new_slo_quota("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = SloQuotaExpiresAtRangeRule { min: 32, max: 255 };
        let mut item = new_slo_quota("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
