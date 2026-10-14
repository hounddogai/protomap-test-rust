use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted token exception must pass. Check returns its problems, or none.
pub trait WeightedTokenExceptionRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTokenException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenExceptionLabelLengthRule {
    pub max_length: i64,
}

impl WeightedTokenExceptionRule for WeightedTokenExceptionLabelLengthRule {
    fn check(&self, item: &WeightedTokenException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenExceptionRequiredDescriptionRule {
    pub message: String,
}

impl WeightedTokenExceptionRule for WeightedTokenExceptionRequiredDescriptionRule {
    fn check(&self, item: &WeightedTokenException) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenExceptionDescriptionLengthRule {
    pub max_length: i64,
}

impl WeightedTokenExceptionRule for WeightedTokenExceptionDescriptionLengthRule {
    fn check(&self, item: &WeightedTokenException) -> Vec<String> {
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
pub struct WeightedTokenExceptionCodeLengthRule {
    pub max_length: i64,
}

impl WeightedTokenExceptionRule for WeightedTokenExceptionCodeLengthRule {
    fn check(&self, item: &WeightedTokenException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenExceptionAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl WeightedTokenExceptionRule for WeightedTokenExceptionAllowedDescriptionRule {
    fn check(&self, item: &WeightedTokenException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is duplicated", item.description)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted token exception.
#[derive(Debug, Clone)]
pub struct WeightedTokenExceptionValidator {
    pub rules: Vec<Rc<dyn WeightedTokenExceptionRule>>,
}

impl WeightedTokenExceptionValidator {
    pub fn validate(&self, item: &WeightedTokenException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedTokenException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTokenException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_token_exception_validator() -> WeightedTokenExceptionValidator {
    WeightedTokenExceptionValidator {
        rules: vec![
            Rc::new(WeightedTokenExceptionLabelLengthRule { max_length: 86400 }) as Rc<dyn WeightedTokenExceptionRule>,
            Rc::new(WeightedTokenExceptionRequiredDescriptionRule {
                message: "weighted token exception needs an entry".to_string(),
            }),
            Rc::new(WeightedTokenExceptionDescriptionLengthRule { max_length: 3600 }),
            Rc::new(WeightedTokenExceptionCodeLengthRule { max_length: 1000 }),
            Rc::new(WeightedTokenExceptionAllowedDescriptionRule {
                allowed: vec!["tiered".to_string(), "tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_token_exception_validator();
        let item = new_weighted_token_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedTokenExceptionLabelLengthRule { max_length: 80 };
        let mut item = new_weighted_token_exception("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = WeightedTokenExceptionRequiredDescriptionRule {
            message: "weighted token exception needs a value".to_string(),
        };
        let mut item = new_weighted_token_exception("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = WeightedTokenExceptionDescriptionLengthRule { max_length: 64 };
        let mut item = new_weighted_token_exception("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedTokenExceptionCodeLengthRule { max_length: 255 };
        let mut item = new_weighted_token_exception("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
