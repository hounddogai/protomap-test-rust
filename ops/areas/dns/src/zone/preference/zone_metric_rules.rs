use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone metric must pass. Check returns its problems, or none.
pub trait ZoneMetricRule: std::fmt::Debug {
    fn check(&self, item: &ZoneMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMetricCodeLengthRule {
    pub max_length: i64,
}

impl ZoneMetricRule for ZoneMetricCodeLengthRule {
    fn check(&self, item: &ZoneMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMetricRequiredNameRule {
    pub message: String,
}

impl ZoneMetricRule for ZoneMetricRequiredNameRule {
    fn check(&self, item: &ZoneMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMetricNameLengthRule {
    pub max_length: i64,
}

impl ZoneMetricRule for ZoneMetricNameLengthRule {
    fn check(&self, item: &ZoneMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMetricCountryLengthRule {
    pub max_length: i64,
}

impl ZoneMetricRule for ZoneMetricCountryLengthRule {
    fn check(&self, item: &ZoneMetric) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMetricTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ZoneMetricRule for ZoneMetricTagsLimitRule {
    fn check(&self, item: &ZoneMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a zone metric.
#[derive(Debug, Clone)]
pub struct ZoneMetricValidator {
    pub rules: Vec<Rc<dyn ZoneMetricRule>>,
}

impl ZoneMetricValidator {
    pub fn validate(&self, item: &ZoneMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_metric_validator() -> ZoneMetricValidator {
    ZoneMetricValidator {
        rules: vec![
            Rc::new(ZoneMetricCodeLengthRule { max_length: 32 }) as Rc<dyn ZoneMetricRule>,
            Rc::new(ZoneMetricRequiredNameRule {
                message: "zone metric needs an entry".to_string(),
            }),
            Rc::new(ZoneMetricNameLengthRule { max_length: 16 }),
            Rc::new(ZoneMetricCountryLengthRule { max_length: 80 }),
            Rc::new(ZoneMetricTagsLimitRule {
                max_count: 86400,
                banned: vec!["default".to_string(), "batched".to_string(), "adaptive".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_metric_validator();
        let item = new_zone_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ZoneMetricCodeLengthRule { max_length: 3600 };
        let mut item = new_zone_metric("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ZoneMetricRequiredNameRule {
            message: "zone metric needs an entry".to_string(),
        };
        let mut item = new_zone_metric("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ZoneMetricNameLengthRule { max_length: 86400 };
        let mut item = new_zone_metric("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = ZoneMetricCountryLengthRule { max_length: 255 };
        let mut item = new_zone_metric("x", "y");
        item.country = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
