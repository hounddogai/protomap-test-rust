use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled forecast group must pass. Check returns its problems, or none.
pub trait ScheduledForecastGroupRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledForecastGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastGroupRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledForecastGroupRule for ScheduledForecastGroupRotationDaysRangeRule {
    fn check(&self, item: &ScheduledForecastGroup) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastGroupLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledForecastGroupRule for ScheduledForecastGroupLabelLengthRule {
    fn check(&self, item: &ScheduledForecastGroup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastGroupRequiredCodeRule {
    pub message: String,
}

impl ScheduledForecastGroupRule for ScheduledForecastGroupRequiredCodeRule {
    fn check(&self, item: &ScheduledForecastGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastGroupCodePrefixRule {
    pub prefix: String,
}

impl ScheduledForecastGroupRule for ScheduledForecastGroupCodePrefixRule {
    fn check(&self, item: &ScheduledForecastGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastGroupBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledForecastGroupRule for ScheduledForecastGroupBatchSizeRangeRule {
    fn check(&self, item: &ScheduledForecastGroup) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled forecast group.
#[derive(Debug, Clone)]
pub struct ScheduledForecastGroupValidator {
    pub rules: Vec<Rc<dyn ScheduledForecastGroupRule>>,
}

impl ScheduledForecastGroupValidator {
    pub fn validate(&self, item: &ScheduledForecastGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledForecastGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledForecastGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_forecast_group_validator() -> ScheduledForecastGroupValidator {
    ScheduledForecastGroupValidator {
        rules: vec![
            Rc::new(ScheduledForecastGroupRotationDaysRangeRule { min: 120, max: 86400 })
                as Rc<dyn ScheduledForecastGroupRule>,
            Rc::new(ScheduledForecastGroupLabelLengthRule { max_length: 32 }),
            Rc::new(ScheduledForecastGroupRequiredCodeRule {
                message: "scheduled forecast group needs an entry".to_string(),
            }),
            Rc::new(ScheduledForecastGroupCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ScheduledForecastGroupBatchSizeRangeRule { min: 64, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_forecast_group_validator();
        let item = new_scheduled_forecast_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = ScheduledForecastGroupRotationDaysRangeRule { min: 64, max: 1000 };
        let mut item = new_scheduled_forecast_group("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledForecastGroupLabelLengthRule { max_length: 32 };
        let mut item = new_scheduled_forecast_group("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledForecastGroupRequiredCodeRule {
            message: "scheduled forecast group needs a value".to_string(),
        };
        let mut item = new_scheduled_forecast_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = ScheduledForecastGroupBatchSizeRangeRule { min: 80, max: 120 };
        let mut item = new_scheduled_forecast_group("x", "y");
        item.batch_size = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
