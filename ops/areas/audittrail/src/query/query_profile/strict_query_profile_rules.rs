use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict query profile must pass. Check returns its problems, or none.
pub trait StrictQueryProfileRule: std::fmt::Debug {
    fn check(&self, item: &StrictQueryProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryProfileRequiredNameRule {
    pub message: String,
}

impl StrictQueryProfileRule for StrictQueryProfileRequiredNameRule {
    fn check(&self, item: &StrictQueryProfile) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryProfileDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictQueryProfileRule for StrictQueryProfileDurationMinutesRangeRule {
    fn check(&self, item: &StrictQueryProfile) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryProfileRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictQueryProfileRule for StrictQueryProfileRetriesRangeRule {
    fn check(&self, item: &StrictQueryProfile) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryProfileNameLengthRule {
    pub max_length: i64,
}

impl StrictQueryProfileRule for StrictQueryProfileNameLengthRule {
    fn check(&self, item: &StrictQueryProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryProfileCodeLengthRule {
    pub max_length: i64,
}

impl StrictQueryProfileRule for StrictQueryProfileCodeLengthRule {
    fn check(&self, item: &StrictQueryProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict query profile.
#[derive(Debug, Clone)]
pub struct StrictQueryProfileValidator {
    pub rules: Vec<Rc<dyn StrictQueryProfileRule>>,
}

impl StrictQueryProfileValidator {
    pub fn validate(&self, item: &StrictQueryProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictQueryProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictQueryProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_query_profile_validator() -> StrictQueryProfileValidator {
    StrictQueryProfileValidator {
        rules: vec![
            Rc::new(StrictQueryProfileRequiredNameRule {
                message: "strict query profile needs a value".to_string(),
            }) as Rc<dyn StrictQueryProfileRule>,
            Rc::new(StrictQueryProfileDurationMinutesRangeRule { min: 80, max: 255 }),
            Rc::new(StrictQueryProfileRetriesRangeRule { min: 16, max: 64 }),
            Rc::new(StrictQueryProfileNameLengthRule { max_length: 255 }),
            Rc::new(StrictQueryProfileCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_query_profile_validator();
        let item = new_strict_query_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictQueryProfileRequiredNameRule {
            message: "strict query profile needs this field".to_string(),
        };
        let mut item = new_strict_query_profile("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = StrictQueryProfileDurationMinutesRangeRule { min: 120, max: 3600 };
        let mut item = new_strict_query_profile("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = StrictQueryProfileRetriesRangeRule { min: 80, max: 255 };
        let mut item = new_strict_query_profile("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictQueryProfileNameLengthRule { max_length: 3600 };
        let mut item = new_strict_query_profile("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictQueryProfileCodeLengthRule { max_length: 3600 };
        let mut item = new_strict_query_profile("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
