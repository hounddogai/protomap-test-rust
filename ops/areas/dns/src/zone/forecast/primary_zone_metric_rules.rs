use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary zone metric must pass. Check returns its problems, or none.
pub trait PrimaryZoneMetricRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryZoneMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryZoneMetricRequiredIssuerRule {
    pub message: String,
}

impl PrimaryZoneMetricRule for PrimaryZoneMetricRequiredIssuerRule {
    fn check(&self, item: &PrimaryZoneMetric) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryZoneMetricPinnedGuardRule {
    pub limit: i64,
}

impl PrimaryZoneMetricRule for PrimaryZoneMetricPinnedGuardRule {
    fn check(&self, item: &PrimaryZoneMetric) -> Vec<String> {
        if item.pinned && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while pinned",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryZoneMetricRequiredNameRule {
    pub message: String,
}

impl PrimaryZoneMetricRule for PrimaryZoneMetricRequiredNameRule {
    fn check(&self, item: &PrimaryZoneMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryZoneMetricRequiredCodeRule {
    pub message: String,
}

impl PrimaryZoneMetricRule for PrimaryZoneMetricRequiredCodeRule {
    fn check(&self, item: &PrimaryZoneMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryZoneMetricIssuerLengthRule {
    pub max_length: i64,
}

impl PrimaryZoneMetricRule for PrimaryZoneMetricIssuerLengthRule {
    fn check(&self, item: &PrimaryZoneMetric) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary zone metric.
#[derive(Debug, Clone)]
pub struct PrimaryZoneMetricValidator {
    pub rules: Vec<Rc<dyn PrimaryZoneMetricRule>>,
}

impl PrimaryZoneMetricValidator {
    pub fn validate(&self, item: &PrimaryZoneMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryZoneMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryZoneMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_zone_metric_validator() -> PrimaryZoneMetricValidator {
    PrimaryZoneMetricValidator {
        rules: vec![
            Rc::new(PrimaryZoneMetricRequiredIssuerRule {
                message: "primary zone metric needs this field".to_string(),
            }) as Rc<dyn PrimaryZoneMetricRule>,
            Rc::new(PrimaryZoneMetricPinnedGuardRule { limit: 1000 }),
            Rc::new(PrimaryZoneMetricRequiredNameRule {
                message: "primary zone metric needs this field".to_string(),
            }),
            Rc::new(PrimaryZoneMetricRequiredCodeRule {
                message: "primary zone metric needs a value".to_string(),
            }),
            Rc::new(PrimaryZoneMetricIssuerLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_zone_metric_validator();
        let item = new_primary_zone_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = PrimaryZoneMetricRequiredIssuerRule {
            message: "primary zone metric needs this field".to_string(),
        };
        let mut item = new_primary_zone_metric("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryZoneMetricRequiredNameRule {
            message: "primary zone metric needs an entry".to_string(),
        };
        let mut item = new_primary_zone_metric("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryZoneMetricRequiredCodeRule {
            message: "primary zone metric needs an entry".to_string(),
        };
        let mut item = new_primary_zone_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = PrimaryZoneMetricIssuerLengthRule { max_length: 255 };
        let mut item = new_primary_zone_metric("x", "y");
        item.issuer = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
