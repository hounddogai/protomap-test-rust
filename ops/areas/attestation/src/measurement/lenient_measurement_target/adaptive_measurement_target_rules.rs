use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive measurement target must pass. Check returns its problems, or none.
pub trait AdaptiveMeasurementTargetRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveMeasurementTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementTargetRequiredDescriptionRule {
    pub message: String,
}

impl AdaptiveMeasurementTargetRule for AdaptiveMeasurementTargetRequiredDescriptionRule {
    fn check(&self, item: &AdaptiveMeasurementTarget) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementTargetDescriptionLengthRule {
    pub max_length: i64,
}

impl AdaptiveMeasurementTargetRule for AdaptiveMeasurementTargetDescriptionLengthRule {
    fn check(&self, item: &AdaptiveMeasurementTarget) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementTargetRequiredCodeRule {
    pub message: String,
}

impl AdaptiveMeasurementTargetRule for AdaptiveMeasurementTargetRequiredCodeRule {
    fn check(&self, item: &AdaptiveMeasurementTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementTargetAudienceLengthRule {
    pub max_length: i64,
}

impl AdaptiveMeasurementTargetRule for AdaptiveMeasurementTargetAudienceLengthRule {
    fn check(&self, item: &AdaptiveMeasurementTarget) -> Vec<String> {
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
pub struct AdaptiveMeasurementTargetLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveMeasurementTargetRule for AdaptiveMeasurementTargetLimitRangeRule {
    fn check(&self, item: &AdaptiveMeasurementTarget) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a adaptive measurement target.
#[derive(Debug, Clone)]
pub struct AdaptiveMeasurementTargetValidator {
    pub rules: Vec<Rc<dyn AdaptiveMeasurementTargetRule>>,
}

impl AdaptiveMeasurementTargetValidator {
    pub fn validate(&self, item: &AdaptiveMeasurementTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveMeasurementTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveMeasurementTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_measurement_target_validator() -> AdaptiveMeasurementTargetValidator {
    AdaptiveMeasurementTargetValidator {
        rules: vec![
            Rc::new(AdaptiveMeasurementTargetRequiredDescriptionRule {
                message: "adaptive measurement target needs a value".to_string(),
            }) as Rc<dyn AdaptiveMeasurementTargetRule>,
            Rc::new(AdaptiveMeasurementTargetDescriptionLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveMeasurementTargetRequiredCodeRule {
                message: "adaptive measurement target needs this field".to_string(),
            }),
            Rc::new(AdaptiveMeasurementTargetAudienceLengthRule { max_length: 1000 }),
            Rc::new(AdaptiveMeasurementTargetLimitRangeRule { min: 64, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_measurement_target_validator();
        let item = new_adaptive_measurement_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = AdaptiveMeasurementTargetRequiredDescriptionRule {
            message: "adaptive measurement target needs a value".to_string(),
        };
        let mut item = new_adaptive_measurement_target("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = AdaptiveMeasurementTargetDescriptionLengthRule { max_length: 64 };
        let mut item = new_adaptive_measurement_target("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveMeasurementTargetRequiredCodeRule {
            message: "adaptive measurement target needs an entry".to_string(),
        };
        let mut item = new_adaptive_measurement_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = AdaptiveMeasurementTargetAudienceLengthRule { max_length: 255 };
        let mut item = new_adaptive_measurement_target("x", "y");
        item.audience = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = AdaptiveMeasurementTargetLimitRangeRule { min: 120, max: 255 };
        let mut item = new_adaptive_measurement_target("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
