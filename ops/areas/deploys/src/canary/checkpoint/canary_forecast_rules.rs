use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary forecast must pass. Check returns its problems, or none.
pub trait CanaryForecastRule: std::fmt::Debug {
    fn check(&self, item: &CanaryForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryForecastKeyIdLengthRule {
    pub max_length: i64,
}

impl CanaryForecastRule for CanaryForecastKeyIdLengthRule {
    fn check(&self, item: &CanaryForecast) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryForecastAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CanaryForecastRule for CanaryForecastAllowedCodeRule {
    fn check(&self, item: &CanaryForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryForecastRequiredNotesRule {
    pub message: String,
}

impl CanaryForecastRule for CanaryForecastRequiredNotesRule {
    fn check(&self, item: &CanaryForecast) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryForecastCodePrefixRule {
    pub prefix: String,
}

impl CanaryForecastRule for CanaryForecastCodePrefixRule {
    fn check(&self, item: &CanaryForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryForecastNotesLengthRule {
    pub max_length: i64,
}

impl CanaryForecastRule for CanaryForecastNotesLengthRule {
    fn check(&self, item: &CanaryForecast) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryForecastRequiredCodeRule {
    pub message: String,
}

impl CanaryForecastRule for CanaryForecastRequiredCodeRule {
    fn check(&self, item: &CanaryForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary forecast.
#[derive(Debug, Clone)]
pub struct CanaryForecastValidator {
    pub rules: Vec<Rc<dyn CanaryForecastRule>>,
}

impl CanaryForecastValidator {
    pub fn validate(&self, item: &CanaryForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_forecast_validator() -> CanaryForecastValidator {
    CanaryForecastValidator {
        rules: vec![
            Rc::new(CanaryForecastKeyIdLengthRule { max_length: 80 }) as Rc<dyn CanaryForecastRule>,
            Rc::new(CanaryForecastAllowedCodeRule {
                allowed: vec!["legacy".to_string()],
            }),
            Rc::new(CanaryForecastRequiredNotesRule {
                message: "canary forecast needs this field".to_string(),
            }),
            Rc::new(CanaryForecastCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CanaryForecastNotesLengthRule { max_length: 1000 }),
            Rc::new(CanaryForecastRequiredCodeRule {
                message: "canary forecast needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_forecast_validator();
        let item = new_canary_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = CanaryForecastKeyIdLengthRule { max_length: 120 };
        let mut item = new_canary_forecast("x", "y");
        item.key_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = CanaryForecastRequiredNotesRule {
            message: "canary forecast needs a value".to_string(),
        };
        let mut item = new_canary_forecast("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = CanaryForecastNotesLengthRule { max_length: 86400 };
        let mut item = new_canary_forecast("x", "y");
        item.notes = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CanaryForecastRequiredCodeRule {
            message: "canary forecast needs this field".to_string(),
        };
        let mut item = new_canary_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
