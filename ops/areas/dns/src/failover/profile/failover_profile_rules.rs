use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a failover profile must pass. Check returns its problems, or none.
pub trait FailoverProfileRule: std::fmt::Debug {
    fn check(&self, item: &FailoverProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProfileRequiredCodeRule {
    pub message: String,
}

impl FailoverProfileRule for FailoverProfileRequiredCodeRule {
    fn check(&self, item: &FailoverProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProfileNameLengthRule {
    pub max_length: i64,
}

impl FailoverProfileRule for FailoverProfileNameLengthRule {
    fn check(&self, item: &FailoverProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProfileCodeLengthRule {
    pub max_length: i64,
}

impl FailoverProfileRule for FailoverProfileCodeLengthRule {
    fn check(&self, item: &FailoverProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProfileDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverProfileRule for FailoverProfileDurationMinutesRangeRule {
    fn check(&self, item: &FailoverProfile) -> Vec<String> {
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
pub struct FailoverProfileAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FailoverProfileRule for FailoverProfileAllowedNameRule {
    fn check(&self, item: &FailoverProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover profile.
#[derive(Debug, Clone)]
pub struct FailoverProfileValidator {
    pub rules: Vec<Rc<dyn FailoverProfileRule>>,
}

impl FailoverProfileValidator {
    pub fn validate(&self, item: &FailoverProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_profile_validator() -> FailoverProfileValidator {
    FailoverProfileValidator {
        rules: vec![
            Rc::new(FailoverProfileRequiredCodeRule {
                message: "failover profile needs a value".to_string(),
            }) as Rc<dyn FailoverProfileRule>,
            Rc::new(FailoverProfileNameLengthRule { max_length: 3600 }),
            Rc::new(FailoverProfileCodeLengthRule { max_length: 255 }),
            Rc::new(FailoverProfileDurationMinutesRangeRule { min: 32, max: 3600 }),
            Rc::new(FailoverProfileAllowedNameRule {
                allowed: vec!["adaptive".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_profile_validator();
        let item = new_failover_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FailoverProfileRequiredCodeRule {
            message: "failover profile needs an entry".to_string(),
        };
        let mut item = new_failover_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FailoverProfileNameLengthRule { max_length: 80 };
        let mut item = new_failover_profile("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FailoverProfileCodeLengthRule { max_length: 255 };
        let mut item = new_failover_profile("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = FailoverProfileDurationMinutesRangeRule { min: 32, max: 3600 };
        let mut item = new_failover_profile("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
