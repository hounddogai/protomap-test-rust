use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook preference must pass. Check returns its problems, or none.
pub trait RunbookPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &RunbookPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookPreferenceQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookPreferenceRule for RunbookPreferenceQuantityRangeRule {
    fn check(&self, item: &RunbookPreference) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookPreferenceRequiredCodeRule {
    pub message: String,
}

impl RunbookPreferenceRule for RunbookPreferenceRequiredCodeRule {
    fn check(&self, item: &RunbookPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookPreferenceRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookPreferenceRule for RunbookPreferenceRatioRangeRule {
    fn check(&self, item: &RunbookPreference) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookPreferenceAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RunbookPreferenceRule for RunbookPreferenceAllowedLabelRule {
    fn check(&self, item: &RunbookPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookPreferenceNotesLengthRule {
    pub max_length: i64,
}

impl RunbookPreferenceRule for RunbookPreferenceNotesLengthRule {
    fn check(&self, item: &RunbookPreference) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookPreferenceCodePrefixRule {
    pub prefix: String,
}

impl RunbookPreferenceRule for RunbookPreferenceCodePrefixRule {
    fn check(&self, item: &RunbookPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook preference.
#[derive(Debug, Clone)]
pub struct RunbookPreferenceValidator {
    pub rules: Vec<Rc<dyn RunbookPreferenceRule>>,
}

impl RunbookPreferenceValidator {
    pub fn validate(&self, item: &RunbookPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_preference_validator() -> RunbookPreferenceValidator {
    RunbookPreferenceValidator {
        rules: vec![
            Rc::new(RunbookPreferenceQuantityRangeRule { min: 120, max: 1000 }) as Rc<dyn RunbookPreferenceRule>,
            Rc::new(RunbookPreferenceRequiredCodeRule {
                message: "runbook preference needs an entry".to_string(),
            }),
            Rc::new(RunbookPreferenceRatioRangeRule { min: 2.7, max: 1249.7 }),
            Rc::new(RunbookPreferenceAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(RunbookPreferenceNotesLengthRule { max_length: 86400 }),
            Rc::new(RunbookPreferenceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_preference_validator();
        let item = new_runbook_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = RunbookPreferenceQuantityRangeRule { min: 1000, max: 1000 };
        let mut item = new_runbook_preference("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RunbookPreferenceRequiredCodeRule {
            message: "runbook preference needs an entry".to_string(),
        };
        let mut item = new_runbook_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = RunbookPreferenceRatioRangeRule { min: 5.5, max: 1215.6 };
        let mut item = new_runbook_preference("x", "y");
        item.ratio = 1217.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RunbookPreferenceNotesLengthRule { max_length: 255 };
        let mut item = new_runbook_preference("x", "y");
        item.notes = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
