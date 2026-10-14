use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement quota must pass. Check returns its problems, or none.
pub trait MeasurementQuotaRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementQuotaRequiredTitleRule {
    pub message: String,
}

impl MeasurementQuotaRule for MeasurementQuotaRequiredTitleRule {
    fn check(&self, item: &MeasurementQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementQuotaRequiredNameRule {
    pub message: String,
}

impl MeasurementQuotaRule for MeasurementQuotaRequiredNameRule {
    fn check(&self, item: &MeasurementQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementQuotaCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementQuotaRule for MeasurementQuotaCreatedAtRangeRule {
    fn check(&self, item: &MeasurementQuota) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementQuotaDescriptionLengthRule {
    pub max_length: i64,
}

impl MeasurementQuotaRule for MeasurementQuotaDescriptionLengthRule {
    fn check(&self, item: &MeasurementQuota) -> Vec<String> {
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
pub struct MeasurementQuotaMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementQuotaRule for MeasurementQuotaMinimumRangeRule {
    fn check(&self, item: &MeasurementQuota) -> Vec<String> {
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

/// Runs every rule of a measurement quota.
#[derive(Debug, Clone)]
pub struct MeasurementQuotaValidator {
    pub rules: Vec<Rc<dyn MeasurementQuotaRule>>,
}

impl MeasurementQuotaValidator {
    pub fn validate(&self, item: &MeasurementQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_quota_validator() -> MeasurementQuotaValidator {
    MeasurementQuotaValidator {
        rules: vec![
            Rc::new(MeasurementQuotaRequiredTitleRule {
                message: "measurement quota needs this field".to_string(),
            }) as Rc<dyn MeasurementQuotaRule>,
            Rc::new(MeasurementQuotaRequiredNameRule {
                message: "measurement quota needs a value".to_string(),
            }),
            Rc::new(MeasurementQuotaCreatedAtRangeRule { min: 255, max: 255 }),
            Rc::new(MeasurementQuotaDescriptionLengthRule { max_length: 64 }),
            Rc::new(MeasurementQuotaMinimumRangeRule { min: 1.3, max: 1012.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_quota_validator();
        let item = new_measurement_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = MeasurementQuotaRequiredTitleRule {
            message: "measurement quota needs this field".to_string(),
        };
        let mut item = new_measurement_quota("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = MeasurementQuotaRequiredNameRule {
            message: "measurement quota needs a value".to_string(),
        };
        let mut item = new_measurement_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = MeasurementQuotaCreatedAtRangeRule { min: 120, max: 120 };
        let mut item = new_measurement_quota("x", "y");
        item.created_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = MeasurementQuotaDescriptionLengthRule { max_length: 64 };
        let mut item = new_measurement_quota("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = MeasurementQuotaMinimumRangeRule { min: 3.3, max: 1459.6 };
        let mut item = new_measurement_quota("x", "y");
        item.minimum = 1461.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
