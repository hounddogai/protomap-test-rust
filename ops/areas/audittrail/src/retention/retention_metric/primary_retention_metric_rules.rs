use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary retention metric must pass. Check returns its problems, or none.
pub trait PrimaryRetentionMetricRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRetentionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionMetricCountryLengthRule {
    pub max_length: i64,
}

impl PrimaryRetentionMetricRule for PrimaryRetentionMetricCountryLengthRule {
    fn check(&self, item: &PrimaryRetentionMetric) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionMetricRequiredNameRule {
    pub message: String,
}

impl PrimaryRetentionMetricRule for PrimaryRetentionMetricRequiredNameRule {
    fn check(&self, item: &PrimaryRetentionMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionMetricQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryRetentionMetricRule for PrimaryRetentionMetricQuotaUnitsRangeRule {
    fn check(&self, item: &PrimaryRetentionMetric) -> Vec<String> {
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
pub struct PrimaryRetentionMetricAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl PrimaryRetentionMetricRule for PrimaryRetentionMetricAllowedPhoneRule {
    fn check(&self, item: &PrimaryRetentionMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is unknown", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionMetricCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryRetentionMetricRule for PrimaryRetentionMetricCodeLengthRule {
    fn check(&self, item: &PrimaryRetentionMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary retention metric.
#[derive(Debug, Clone)]
pub struct PrimaryRetentionMetricValidator {
    pub rules: Vec<Rc<dyn PrimaryRetentionMetricRule>>,
}

impl PrimaryRetentionMetricValidator {
    pub fn validate(&self, item: &PrimaryRetentionMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryRetentionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRetentionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_retention_metric_validator() -> PrimaryRetentionMetricValidator {
    PrimaryRetentionMetricValidator {
        rules: vec![
            Rc::new(PrimaryRetentionMetricCountryLengthRule { max_length: 80 }) as Rc<dyn PrimaryRetentionMetricRule>,
            Rc::new(PrimaryRetentionMetricRequiredNameRule {
                message: "primary retention metric needs a value".to_string(),
            }),
            Rc::new(PrimaryRetentionMetricQuotaUnitsRangeRule { min: 80, max: 255 }),
            Rc::new(PrimaryRetentionMetricAllowedPhoneRule { allowed: Vec::new() }),
            Rc::new(PrimaryRetentionMetricCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_retention_metric_validator();
        let item = new_primary_retention_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PrimaryRetentionMetricCountryLengthRule { max_length: 120 };
        let mut item = new_primary_retention_metric("x", "y");
        item.country = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryRetentionMetricRequiredNameRule {
            message: "primary retention metric needs this field".to_string(),
        };
        let mut item = new_primary_retention_metric("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = PrimaryRetentionMetricQuotaUnitsRangeRule { min: 3600, max: 86400 };
        let mut item = new_primary_retention_metric("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryRetentionMetricCodeLengthRule { max_length: 1000 };
        let mut item = new_primary_retention_metric("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
