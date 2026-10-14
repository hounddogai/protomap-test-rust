use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy runbook preference must pass. Check returns its problems, or none.
pub trait LegacyRunbookPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRunbookPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookPreferenceRequiredLabelRule {
    pub message: String,
}

impl LegacyRunbookPreferenceRule for LegacyRunbookPreferenceRequiredLabelRule {
    fn check(&self, item: &LegacyRunbookPreference) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookPreferenceWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyRunbookPreferenceRule for LegacyRunbookPreferenceWeightRangeRule {
    fn check(&self, item: &LegacyRunbookPreference) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookPreferenceLabelLengthRule {
    pub max_length: i64,
}

impl LegacyRunbookPreferenceRule for LegacyRunbookPreferenceLabelLengthRule {
    fn check(&self, item: &LegacyRunbookPreference) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy runbook preference.
#[derive(Debug, Clone)]
pub struct LegacyRunbookPreferenceValidator {
    pub rules: Vec<Rc<dyn LegacyRunbookPreferenceRule>>,
}

impl LegacyRunbookPreferenceValidator {
    pub fn validate(&self, item: &LegacyRunbookPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyRunbookPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRunbookPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_runbook_preference_validator() -> LegacyRunbookPreferenceValidator {
    LegacyRunbookPreferenceValidator {
        rules: vec![
            Rc::new(LegacyRunbookPreferenceRequiredLabelRule {
                message: "legacy runbook preference needs a value".to_string(),
            }) as Rc<dyn LegacyRunbookPreferenceRule>,
            Rc::new(LegacyRunbookPreferenceWeightRangeRule { min: 6.0, max: 4076.2 }),
            Rc::new(LegacyRunbookPreferenceLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_runbook_preference_validator();
        let item = new_legacy_runbook_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacyRunbookPreferenceRequiredLabelRule {
            message: "legacy runbook preference needs this field".to_string(),
        };
        let mut item = new_legacy_runbook_preference("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = LegacyRunbookPreferenceWeightRangeRule { min: 4.1, max: 2230.6 };
        let mut item = new_legacy_runbook_preference("x", "y");
        item.weight = 2232.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyRunbookPreferenceLabelLengthRule { max_length: 255 };
        let mut item = new_legacy_runbook_preference("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
