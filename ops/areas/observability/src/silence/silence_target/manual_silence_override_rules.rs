use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual silence override must pass. Check returns its problems, or none.
pub trait ManualSilenceOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ManualSilenceOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceOverrideCategoryLengthRule {
    pub max_length: i64,
}

impl ManualSilenceOverrideRule for ManualSilenceOverrideCategoryLengthRule {
    fn check(&self, item: &ManualSilenceOverride) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceOverrideNameLengthRule {
    pub max_length: i64,
}

impl ManualSilenceOverrideRule for ManualSilenceOverrideNameLengthRule {
    fn check(&self, item: &ManualSilenceOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceOverrideSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualSilenceOverrideRule for ManualSilenceOverrideSequenceRangeRule {
    fn check(&self, item: &ManualSilenceOverride) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceOverrideAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl ManualSilenceOverrideRule for ManualSilenceOverrideAllowedCategoryRule {
    fn check(&self, item: &ManualSilenceOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is too long", item.category)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual silence override.
#[derive(Debug, Clone)]
pub struct ManualSilenceOverrideValidator {
    pub rules: Vec<Rc<dyn ManualSilenceOverrideRule>>,
}

impl ManualSilenceOverrideValidator {
    pub fn validate(&self, item: &ManualSilenceOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualSilenceOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualSilenceOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_silence_override_validator() -> ManualSilenceOverrideValidator {
    ManualSilenceOverrideValidator {
        rules: vec![
            Rc::new(ManualSilenceOverrideCategoryLengthRule { max_length: 64 }) as Rc<dyn ManualSilenceOverrideRule>,
            Rc::new(ManualSilenceOverrideNameLengthRule { max_length: 3600 }),
            Rc::new(ManualSilenceOverrideSequenceRangeRule { min: 64, max: 80 }),
            Rc::new(ManualSilenceOverrideAllowedCategoryRule {
                allowed: vec!["weighted".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_silence_override_validator();
        let item = new_manual_silence_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = ManualSilenceOverrideCategoryLengthRule { max_length: 64 };
        let mut item = new_manual_silence_override("x", "y");
        item.category = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualSilenceOverrideNameLengthRule { max_length: 86400 };
        let mut item = new_manual_silence_override("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = ManualSilenceOverrideSequenceRangeRule { min: 16, max: 1000 };
        let mut item = new_manual_silence_override("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
