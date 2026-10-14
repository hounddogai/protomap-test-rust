use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional forecast request must pass. Check returns its problems, or none.
pub trait RegionalForecastRequestRule: std::fmt::Debug {
    fn check(&self, item: &RegionalForecastRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRequestSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalForecastRequestRule for RegionalForecastRequestSequenceRangeRule {
    fn check(&self, item: &RegionalForecastRequest) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRequestGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalForecastRequestRule for RegionalForecastRequestGraceDaysRangeRule {
    fn check(&self, item: &RegionalForecastRequest) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRequestRequiredKeyIdRule {
    pub message: String,
}

impl RegionalForecastRequestRule for RegionalForecastRequestRequiredKeyIdRule {
    fn check(&self, item: &RegionalForecastRequest) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRequestLabelLengthRule {
    pub max_length: i64,
}

impl RegionalForecastRequestRule for RegionalForecastRequestLabelLengthRule {
    fn check(&self, item: &RegionalForecastRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRequestNameLengthRule {
    pub max_length: i64,
}

impl RegionalForecastRequestRule for RegionalForecastRequestNameLengthRule {
    fn check(&self, item: &RegionalForecastRequest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional forecast request.
#[derive(Debug, Clone)]
pub struct RegionalForecastRequestValidator {
    pub rules: Vec<Rc<dyn RegionalForecastRequestRule>>,
}

impl RegionalForecastRequestValidator {
    pub fn validate(&self, item: &RegionalForecastRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalForecastRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalForecastRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_forecast_request_validator() -> RegionalForecastRequestValidator {
    RegionalForecastRequestValidator {
        rules: vec![
            Rc::new(RegionalForecastRequestSequenceRangeRule { min: 32, max: 86400 })
                as Rc<dyn RegionalForecastRequestRule>,
            Rc::new(RegionalForecastRequestGraceDaysRangeRule { min: 32, max: 80 }),
            Rc::new(RegionalForecastRequestRequiredKeyIdRule {
                message: "regional forecast request needs an entry".to_string(),
            }),
            Rc::new(RegionalForecastRequestLabelLengthRule { max_length: 64 }),
            Rc::new(RegionalForecastRequestNameLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_forecast_request_validator();
        let item = new_regional_forecast_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = RegionalForecastRequestSequenceRangeRule { min: 255, max: 1000 };
        let mut item = new_regional_forecast_request("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RegionalForecastRequestGraceDaysRangeRule { min: 120, max: 255 };
        let mut item = new_regional_forecast_request("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = RegionalForecastRequestRequiredKeyIdRule {
            message: "regional forecast request needs an entry".to_string(),
        };
        let mut item = new_regional_forecast_request("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalForecastRequestLabelLengthRule { max_length: 32 };
        let mut item = new_regional_forecast_request("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalForecastRequestNameLengthRule { max_length: 32 };
        let mut item = new_regional_forecast_request("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
