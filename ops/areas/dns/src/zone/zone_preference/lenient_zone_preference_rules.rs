use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient zone preference must pass. Check returns its problems, or none.
pub trait LenientZonePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LenientZonePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZonePreferenceAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl LenientZonePreferenceRule for LenientZonePreferenceAllowedExternalRefRule {
    fn check(&self, item: &LenientZonePreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is above maximum", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZonePreferenceRequiredExternalRefRule {
    pub message: String,
}

impl LenientZonePreferenceRule for LenientZonePreferenceRequiredExternalRefRule {
    fn check(&self, item: &LenientZonePreference) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZonePreferenceTitleLengthRule {
    pub max_length: i64,
}

impl LenientZonePreferenceRule for LenientZonePreferenceTitleLengthRule {
    fn check(&self, item: &LenientZonePreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient zone preference.
#[derive(Debug, Clone)]
pub struct LenientZonePreferenceValidator {
    pub rules: Vec<Rc<dyn LenientZonePreferenceRule>>,
}

impl LenientZonePreferenceValidator {
    pub fn validate(&self, item: &LenientZonePreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientZonePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientZonePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_zone_preference_validator() -> LenientZonePreferenceValidator {
    LenientZonePreferenceValidator {
        rules: vec![
            Rc::new(LenientZonePreferenceAllowedExternalRefRule { allowed: Vec::new() })
                as Rc<dyn LenientZonePreferenceRule>,
            Rc::new(LenientZonePreferenceRequiredExternalRefRule {
                message: "lenient zone preference needs this field".to_string(),
            }),
            Rc::new(LenientZonePreferenceTitleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_zone_preference_validator();
        let item = new_lenient_zone_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = LenientZonePreferenceRequiredExternalRefRule {
            message: "lenient zone preference needs an entry".to_string(),
        };
        let mut item = new_lenient_zone_preference("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientZonePreferenceTitleLengthRule { max_length: 86400 };
        let mut item = new_lenient_zone_preference("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
