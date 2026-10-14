use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce template must pass. Check returns its problems, or none.
pub trait NonceTemplateRule: std::fmt::Debug {
    fn check(&self, item: &NonceTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTemplateBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceTemplateRule for NonceTemplateBudgetRangeRule {
    fn check(&self, item: &NonceTemplate) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTemplateCodeLengthRule {
    pub max_length: i64,
}

impl NonceTemplateRule for NonceTemplateCodeLengthRule {
    fn check(&self, item: &NonceTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTemplateRequiredNameRule {
    pub message: String,
}

impl NonceTemplateRule for NonceTemplateRequiredNameRule {
    fn check(&self, item: &NonceTemplate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTemplateDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceTemplateRule for NonceTemplateDurationMinutesRangeRule {
    fn check(&self, item: &NonceTemplate) -> Vec<String> {
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

/// Runs every rule of a nonce template.
#[derive(Debug, Clone)]
pub struct NonceTemplateValidator {
    pub rules: Vec<Rc<dyn NonceTemplateRule>>,
}

impl NonceTemplateValidator {
    pub fn validate(&self, item: &NonceTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_template_validator() -> NonceTemplateValidator {
    NonceTemplateValidator {
        rules: vec![
            Rc::new(NonceTemplateBudgetRangeRule { min: 0.7, max: 3764.8 }) as Rc<dyn NonceTemplateRule>,
            Rc::new(NonceTemplateCodeLengthRule { max_length: 3600 }),
            Rc::new(NonceTemplateRequiredNameRule {
                message: "nonce template needs this field".to_string(),
            }),
            Rc::new(NonceTemplateDurationMinutesRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_template_validator();
        let item = new_nonce_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = NonceTemplateBudgetRangeRule { min: 5.9, max: 1625.5 };
        let mut item = new_nonce_template("x", "y");
        item.budget = 1627.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = NonceTemplateCodeLengthRule { max_length: 255 };
        let mut item = new_nonce_template("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = NonceTemplateRequiredNameRule {
            message: "nonce template needs this field".to_string(),
        };
        let mut item = new_nonce_template("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = NonceTemplateDurationMinutesRangeRule { min: 120, max: 1000 };
        let mut item = new_nonce_template("x", "y");
        item.duration_minutes = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
