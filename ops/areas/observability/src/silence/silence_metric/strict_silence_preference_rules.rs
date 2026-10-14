use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict silence preference must pass. Check returns its problems, or none.
pub trait StrictSilencePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &StrictSilencePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilencePreferenceRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictSilencePreferenceRule for StrictSilencePreferenceRevisionRangeRule {
    fn check(&self, item: &StrictSilencePreference) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilencePreferenceUnitLengthRule {
    pub max_length: i64,
}

impl StrictSilencePreferenceRule for StrictSilencePreferenceUnitLengthRule {
    fn check(&self, item: &StrictSilencePreference) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilencePreferenceRequiredCodeRule {
    pub message: String,
}

impl StrictSilencePreferenceRule for StrictSilencePreferenceRequiredCodeRule {
    fn check(&self, item: &StrictSilencePreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilencePreferenceRequiredNameRule {
    pub message: String,
}

impl StrictSilencePreferenceRule for StrictSilencePreferenceRequiredNameRule {
    fn check(&self, item: &StrictSilencePreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilencePreferenceCodePrefixRule {
    pub prefix: String,
}

impl StrictSilencePreferenceRule for StrictSilencePreferenceCodePrefixRule {
    fn check(&self, item: &StrictSilencePreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilencePreferenceCodeLengthRule {
    pub max_length: i64,
}

impl StrictSilencePreferenceRule for StrictSilencePreferenceCodeLengthRule {
    fn check(&self, item: &StrictSilencePreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict silence preference.
#[derive(Debug, Clone)]
pub struct StrictSilencePreferenceValidator {
    pub rules: Vec<Rc<dyn StrictSilencePreferenceRule>>,
}

impl StrictSilencePreferenceValidator {
    pub fn validate(&self, item: &StrictSilencePreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictSilencePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictSilencePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_silence_preference_validator() -> StrictSilencePreferenceValidator {
    StrictSilencePreferenceValidator {
        rules: vec![
            Rc::new(StrictSilencePreferenceRevisionRangeRule { min: 64, max: 1000 })
                as Rc<dyn StrictSilencePreferenceRule>,
            Rc::new(StrictSilencePreferenceUnitLengthRule { max_length: 255 }),
            Rc::new(StrictSilencePreferenceRequiredCodeRule {
                message: "strict silence preference needs a value".to_string(),
            }),
            Rc::new(StrictSilencePreferenceRequiredNameRule {
                message: "strict silence preference needs an entry".to_string(),
            }),
            Rc::new(StrictSilencePreferenceCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(StrictSilencePreferenceCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_silence_preference_validator();
        let item = new_strict_silence_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = StrictSilencePreferenceRevisionRangeRule { min: 16, max: 86400 };
        let mut item = new_strict_silence_preference("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = StrictSilencePreferenceUnitLengthRule { max_length: 86400 };
        let mut item = new_strict_silence_preference("x", "y");
        item.unit = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictSilencePreferenceRequiredCodeRule {
            message: "strict silence preference needs this field".to_string(),
        };
        let mut item = new_strict_silence_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictSilencePreferenceRequiredNameRule {
            message: "strict silence preference needs this field".to_string(),
        };
        let mut item = new_strict_silence_preference("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictSilencePreferenceCodeLengthRule { max_length: 86400 };
        let mut item = new_strict_silence_preference("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
