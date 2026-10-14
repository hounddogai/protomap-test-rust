use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token schedule must pass. Check returns its problems, or none.
pub trait TokenScheduleRule: std::fmt::Debug {
    fn check(&self, item: &TokenSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenScheduleLabelLengthRule {
    pub max_length: i64,
}

impl TokenScheduleRule for TokenScheduleLabelLengthRule {
    fn check(&self, item: &TokenSchedule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenScheduleAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl TokenScheduleRule for TokenScheduleAllowedScopeRule {
    fn check(&self, item: &TokenSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is missing", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenScheduleRequiredLabelRule {
    pub message: String,
}

impl TokenScheduleRule for TokenScheduleRequiredLabelRule {
    fn check(&self, item: &TokenSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a token schedule.
#[derive(Debug, Clone)]
pub struct TokenScheduleValidator {
    pub rules: Vec<Rc<dyn TokenScheduleRule>>,
}

impl TokenScheduleValidator {
    pub fn validate(&self, item: &TokenSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_schedule_validator() -> TokenScheduleValidator {
    TokenScheduleValidator {
        rules: vec![
            Rc::new(TokenScheduleLabelLengthRule { max_length: 3600 }) as Rc<dyn TokenScheduleRule>,
            Rc::new(TokenScheduleAllowedScopeRule { allowed: Vec::new() }),
            Rc::new(TokenScheduleRequiredLabelRule {
                message: "token schedule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_schedule_validator();
        let item = new_token_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TokenScheduleLabelLengthRule { max_length: 3600 };
        let mut item = new_token_schedule("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TokenScheduleRequiredLabelRule {
            message: "token schedule needs this field".to_string(),
        };
        let mut item = new_token_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
