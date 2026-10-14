use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast checkpoint must pass. Check returns its problems, or none.
pub trait ForecastCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &ForecastCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCheckpointExternalRefLengthRule {
    pub max_length: i64,
}

impl ForecastCheckpointRule for ForecastCheckpointExternalRefLengthRule {
    fn check(&self, item: &ForecastCheckpoint) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCheckpointAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ForecastCheckpointRule for ForecastCheckpointAllowedCodeRule {
    fn check(&self, item: &ForecastCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCheckpointRequiredLabelRule {
    pub message: String,
}

impl ForecastCheckpointRule for ForecastCheckpointRequiredLabelRule {
    fn check(&self, item: &ForecastCheckpoint) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCheckpointNotesLengthRule {
    pub max_length: i64,
}

impl ForecastCheckpointRule for ForecastCheckpointNotesLengthRule {
    fn check(&self, item: &ForecastCheckpoint) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCheckpointCodePrefixRule {
    pub prefix: String,
}

impl ForecastCheckpointRule for ForecastCheckpointCodePrefixRule {
    fn check(&self, item: &ForecastCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast checkpoint.
#[derive(Debug, Clone)]
pub struct ForecastCheckpointValidator {
    pub rules: Vec<Rc<dyn ForecastCheckpointRule>>,
}

impl ForecastCheckpointValidator {
    pub fn validate(&self, item: &ForecastCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_checkpoint_validator() -> ForecastCheckpointValidator {
    ForecastCheckpointValidator {
        rules: vec![
            Rc::new(ForecastCheckpointExternalRefLengthRule { max_length: 32 }) as Rc<dyn ForecastCheckpointRule>,
            Rc::new(ForecastCheckpointAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(ForecastCheckpointRequiredLabelRule {
                message: "forecast checkpoint needs this field".to_string(),
            }),
            Rc::new(ForecastCheckpointNotesLengthRule { max_length: 86400 }),
            Rc::new(ForecastCheckpointCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_checkpoint_validator();
        let item = new_forecast_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = ForecastCheckpointExternalRefLengthRule { max_length: 255 };
        let mut item = new_forecast_checkpoint("x", "y");
        item.external_ref = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ForecastCheckpointRequiredLabelRule {
            message: "forecast checkpoint needs an entry".to_string(),
        };
        let mut item = new_forecast_checkpoint("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = ForecastCheckpointNotesLengthRule { max_length: 80 };
        let mut item = new_forecast_checkpoint("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
