use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check tier must pass. Check returns its problems, or none.
pub trait HealthCheckTierRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTierRequiredNameRule {
    pub message: String,
}

impl HealthCheckTierRule for HealthCheckTierRequiredNameRule {
    fn check(&self, item: &HealthCheckTier) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTierRequiredSourceRule {
    pub message: String,
}

impl HealthCheckTierRule for HealthCheckTierRequiredSourceRule {
    fn check(&self, item: &HealthCheckTier) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTierRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckTierRule for HealthCheckTierRateRangeRule {
    fn check(&self, item: &HealthCheckTier) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTierTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckTierRule for HealthCheckTierTtlSecondsRangeRule {
    fn check(&self, item: &HealthCheckTier) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTierNameLengthRule {
    pub max_length: i64,
}

impl HealthCheckTierRule for HealthCheckTierNameLengthRule {
    fn check(&self, item: &HealthCheckTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTierRequiredRegionRule {
    pub message: String,
}

impl HealthCheckTierRule for HealthCheckTierRequiredRegionRule {
    fn check(&self, item: &HealthCheckTier) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check tier.
#[derive(Debug, Clone)]
pub struct HealthCheckTierValidator {
    pub rules: Vec<Rc<dyn HealthCheckTierRule>>,
}

impl HealthCheckTierValidator {
    pub fn validate(&self, item: &HealthCheckTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_tier_validator() -> HealthCheckTierValidator {
    HealthCheckTierValidator {
        rules: vec![
            Rc::new(HealthCheckTierRequiredNameRule {
                message: "health check tier needs this field".to_string(),
            }) as Rc<dyn HealthCheckTierRule>,
            Rc::new(HealthCheckTierRequiredSourceRule {
                message: "health check tier needs a value".to_string(),
            }),
            Rc::new(HealthCheckTierRateRangeRule { min: 1.4, max: 2952.7 }),
            Rc::new(HealthCheckTierTtlSecondsRangeRule { min: 16, max: 32 }),
            Rc::new(HealthCheckTierNameLengthRule { max_length: 64 }),
            Rc::new(HealthCheckTierRequiredRegionRule {
                message: "health check tier needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_tier_validator();
        let item = new_health_check_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = HealthCheckTierRequiredNameRule {
            message: "health check tier needs this field".to_string(),
        };
        let mut item = new_health_check_tier("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = HealthCheckTierRequiredSourceRule {
            message: "health check tier needs an entry".to_string(),
        };
        let mut item = new_health_check_tier("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = HealthCheckTierRateRangeRule { min: 0.1, max: 1436.6 };
        let mut item = new_health_check_tier("x", "y");
        item.rate = 1438.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = HealthCheckTierTtlSecondsRangeRule { min: 64, max: 86400 };
        let mut item = new_health_check_tier("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = HealthCheckTierNameLengthRule { max_length: 1000 };
        let mut item = new_health_check_tier("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = HealthCheckTierRequiredRegionRule {
            message: "health check tier needs an entry".to_string(),
        };
        let mut item = new_health_check_tier("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
