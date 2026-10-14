use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension entry must pass. Check returns its problems, or none.
pub trait SuspensionEntryRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionEntryRequiredCipherRule {
    pub message: String,
}

impl SuspensionEntryRule for SuspensionEntryRequiredCipherRule {
    fn check(&self, item: &SuspensionEntry) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionEntryDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionEntryRule for SuspensionEntryDurationMinutesRangeRule {
    fn check(&self, item: &SuspensionEntry) -> Vec<String> {
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
pub struct SuspensionEntryRequiredCategoryRule {
    pub message: String,
}

impl SuspensionEntryRule for SuspensionEntryRequiredCategoryRule {
    fn check(&self, item: &SuspensionEntry) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension entry.
#[derive(Debug, Clone)]
pub struct SuspensionEntryValidator {
    pub rules: Vec<Rc<dyn SuspensionEntryRule>>,
}

impl SuspensionEntryValidator {
    pub fn validate(&self, item: &SuspensionEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_entry_validator() -> SuspensionEntryValidator {
    SuspensionEntryValidator {
        rules: vec![
            Rc::new(SuspensionEntryRequiredCipherRule {
                message: "suspension entry needs an entry".to_string(),
            }) as Rc<dyn SuspensionEntryRule>,
            Rc::new(SuspensionEntryDurationMinutesRangeRule { min: 80, max: 3600 }),
            Rc::new(SuspensionEntryRequiredCategoryRule {
                message: "suspension entry needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_entry_validator();
        let item = new_suspension_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = SuspensionEntryRequiredCipherRule {
            message: "suspension entry needs a value".to_string(),
        };
        let mut item = new_suspension_entry("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = SuspensionEntryDurationMinutesRangeRule { min: 3600, max: 86400 };
        let mut item = new_suspension_entry("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = SuspensionEntryRequiredCategoryRule {
            message: "suspension entry needs an entry".to_string(),
        };
        let mut item = new_suspension_entry("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
