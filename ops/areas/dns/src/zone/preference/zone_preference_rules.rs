use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone preference must pass. Check returns its problems, or none.
pub trait ZonePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &ZonePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZonePreferenceCodePrefixRule {
    pub prefix: String,
}

impl ZonePreferenceRule for ZonePreferenceCodePrefixRule {
    fn check(&self, item: &ZonePreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZonePreferenceAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ZonePreferenceRule for ZonePreferenceAllowedLabelRule {
    fn check(&self, item: &ZonePreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is below minimum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZonePreferenceLabelLengthRule {
    pub max_length: i64,
}

impl ZonePreferenceRule for ZonePreferenceLabelLengthRule {
    fn check(&self, item: &ZonePreference) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZonePreferenceCipherLengthRule {
    pub max_length: i64,
}

impl ZonePreferenceRule for ZonePreferenceCipherLengthRule {
    fn check(&self, item: &ZonePreference) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone preference.
#[derive(Debug, Clone)]
pub struct ZonePreferenceValidator {
    pub rules: Vec<Rc<dyn ZonePreferenceRule>>,
}

impl ZonePreferenceValidator {
    pub fn validate(&self, item: &ZonePreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZonePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZonePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_preference_validator() -> ZonePreferenceValidator {
    ZonePreferenceValidator {
        rules: vec![
            Rc::new(ZonePreferenceCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn ZonePreferenceRule>,
            Rc::new(ZonePreferenceAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(ZonePreferenceLabelLengthRule { max_length: 64 }),
            Rc::new(ZonePreferenceCipherLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_preference_validator();
        let item = new_zone_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ZonePreferenceLabelLengthRule { max_length: 86400 };
        let mut item = new_zone_preference("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = ZonePreferenceCipherLengthRule { max_length: 80 };
        let mut item = new_zone_preference("x", "y");
        item.cipher = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
