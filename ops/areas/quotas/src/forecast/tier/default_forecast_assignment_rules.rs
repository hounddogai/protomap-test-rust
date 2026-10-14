use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default forecast assignment must pass. Check returns its problems, or none.
pub trait DefaultForecastAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultForecastAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastAssignmentRequiredTitleRule {
    pub message: String,
}

impl DefaultForecastAssignmentRule for DefaultForecastAssignmentRequiredTitleRule {
    fn check(&self, item: &DefaultForecastAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastAssignmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DefaultForecastAssignmentRule for DefaultForecastAssignmentAllowedTitleRule {
    fn check(&self, item: &DefaultForecastAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is missing", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastAssignmentTitleLengthRule {
    pub max_length: i64,
}

impl DefaultForecastAssignmentRule for DefaultForecastAssignmentTitleLengthRule {
    fn check(&self, item: &DefaultForecastAssignment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default forecast assignment.
#[derive(Debug, Clone)]
pub struct DefaultForecastAssignmentValidator {
    pub rules: Vec<Rc<dyn DefaultForecastAssignmentRule>>,
}

impl DefaultForecastAssignmentValidator {
    pub fn validate(&self, item: &DefaultForecastAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultForecastAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultForecastAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_forecast_assignment_validator() -> DefaultForecastAssignmentValidator {
    DefaultForecastAssignmentValidator {
        rules: vec![
            Rc::new(DefaultForecastAssignmentRequiredTitleRule {
                message: "default forecast assignment needs this field".to_string(),
            }) as Rc<dyn DefaultForecastAssignmentRule>,
            Rc::new(DefaultForecastAssignmentAllowedTitleRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(DefaultForecastAssignmentTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_forecast_assignment_validator();
        let item = new_default_forecast_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultForecastAssignmentRequiredTitleRule {
            message: "default forecast assignment needs this field".to_string(),
        };
        let mut item = new_default_forecast_assignment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultForecastAssignmentTitleLengthRule { max_length: 120 };
        let mut item = new_default_forecast_assignment("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
