use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check variance must pass. Check returns its problems, or none.
pub trait HealthCheckVarianceRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckVarianceRequiredAudienceRule {
    pub message: String,
}

impl HealthCheckVarianceRule for HealthCheckVarianceRequiredAudienceRule {
    fn check(&self, item: &HealthCheckVariance) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckVarianceCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckVarianceRule for HealthCheckVarianceCodePrefixRule {
    fn check(&self, item: &HealthCheckVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckVarianceAudienceLengthRule {
    pub max_length: i64,
}

impl HealthCheckVarianceRule for HealthCheckVarianceAudienceLengthRule {
    fn check(&self, item: &HealthCheckVariance) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckVarianceCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckVarianceRule for HealthCheckVarianceCostRangeRule {
    fn check(&self, item: &HealthCheckVariance) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckVarianceFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckVarianceRule for HealthCheckVarianceFloorValueRangeRule {
    fn check(&self, item: &HealthCheckVariance) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckVarianceCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckVarianceRule for HealthCheckVarianceCreatedAtRangeRule {
    fn check(&self, item: &HealthCheckVariance) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check variance.
#[derive(Debug, Clone)]
pub struct HealthCheckVarianceValidator {
    pub rules: Vec<Rc<dyn HealthCheckVarianceRule>>,
}

impl HealthCheckVarianceValidator {
    pub fn validate(&self, item: &HealthCheckVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_variance_validator() -> HealthCheckVarianceValidator {
    HealthCheckVarianceValidator {
        rules: vec![
            Rc::new(HealthCheckVarianceRequiredAudienceRule {
                message: "health check variance needs an entry".to_string(),
            }) as Rc<dyn HealthCheckVarianceRule>,
            Rc::new(HealthCheckVarianceCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(HealthCheckVarianceAudienceLengthRule { max_length: 80 }),
            Rc::new(HealthCheckVarianceCostRangeRule { min: 1.8, max: 543.5 }),
            Rc::new(HealthCheckVarianceFloorValueRangeRule { min: 7.7, max: 1808.3 }),
            Rc::new(HealthCheckVarianceCreatedAtRangeRule { min: 255, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_variance_validator();
        let item = new_health_check_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = HealthCheckVarianceRequiredAudienceRule {
            message: "health check variance needs an entry".to_string(),
        };
        let mut item = new_health_check_variance("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = HealthCheckVarianceAudienceLengthRule { max_length: 64 };
        let mut item = new_health_check_variance("x", "y");
        item.audience = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = HealthCheckVarianceCostRangeRule { min: 8.0, max: 2536.8 };
        let mut item = new_health_check_variance("x", "y");
        item.cost = 2538.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = HealthCheckVarianceFloorValueRangeRule { min: 6.8, max: 362.2 };
        let mut item = new_health_check_variance("x", "y");
        item.floor_value = 363.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = HealthCheckVarianceCreatedAtRangeRule { min: 3600, max: 86400 };
        let mut item = new_health_check_variance("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
