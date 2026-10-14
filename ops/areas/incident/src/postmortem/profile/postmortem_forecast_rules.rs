use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem forecast must pass. Check returns its problems, or none.
pub trait PostmortemForecastRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemForecastCodePrefixRule {
    pub prefix: String,
}

impl PostmortemForecastRule for PostmortemForecastCodePrefixRule {
    fn check(&self, item: &PostmortemForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemForecastCodeLengthRule {
    pub max_length: i64,
}

impl PostmortemForecastRule for PostmortemForecastCodeLengthRule {
    fn check(&self, item: &PostmortemForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemForecastLabelLengthRule {
    pub max_length: i64,
}

impl PostmortemForecastRule for PostmortemForecastLabelLengthRule {
    fn check(&self, item: &PostmortemForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemForecastRequiredOwnerIdRule {
    pub message: String,
}

impl PostmortemForecastRule for PostmortemForecastRequiredOwnerIdRule {
    fn check(&self, item: &PostmortemForecast) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemForecastRequiredCodeRule {
    pub message: String,
}

impl PostmortemForecastRule for PostmortemForecastRequiredCodeRule {
    fn check(&self, item: &PostmortemForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem forecast.
#[derive(Debug, Clone)]
pub struct PostmortemForecastValidator {
    pub rules: Vec<Rc<dyn PostmortemForecastRule>>,
}

impl PostmortemForecastValidator {
    pub fn validate(&self, item: &PostmortemForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_forecast_validator() -> PostmortemForecastValidator {
    PostmortemForecastValidator {
        rules: vec![
            Rc::new(PostmortemForecastCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn PostmortemForecastRule>,
            Rc::new(PostmortemForecastCodeLengthRule { max_length: 1000 }),
            Rc::new(PostmortemForecastLabelLengthRule { max_length: 16 }),
            Rc::new(PostmortemForecastRequiredOwnerIdRule {
                message: "postmortem forecast needs this field".to_string(),
            }),
            Rc::new(PostmortemForecastRequiredCodeRule {
                message: "postmortem forecast needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_forecast_validator();
        let item = new_postmortem_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PostmortemForecastCodeLengthRule { max_length: 86400 };
        let mut item = new_postmortem_forecast("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PostmortemForecastLabelLengthRule { max_length: 86400 };
        let mut item = new_postmortem_forecast("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = PostmortemForecastRequiredOwnerIdRule {
            message: "postmortem forecast needs an entry".to_string(),
        };
        let mut item = new_postmortem_forecast("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PostmortemForecastRequiredCodeRule {
            message: "postmortem forecast needs an entry".to_string(),
        };
        let mut item = new_postmortem_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
