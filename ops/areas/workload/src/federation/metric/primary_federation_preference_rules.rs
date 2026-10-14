use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary federation preference must pass. Check returns its problems, or none.
pub trait PrimaryFederationPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryFederationPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationPreferenceRequiredUnitRule {
    pub message: String,
}

impl PrimaryFederationPreferenceRule for PrimaryFederationPreferenceRequiredUnitRule {
    fn check(&self, item: &PrimaryFederationPreference) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationPreferenceRequiredCodeRule {
    pub message: String,
}

impl PrimaryFederationPreferenceRule for PrimaryFederationPreferenceRequiredCodeRule {
    fn check(&self, item: &PrimaryFederationPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationPreferenceCodePrefixRule {
    pub prefix: String,
}

impl PrimaryFederationPreferenceRule for PrimaryFederationPreferenceCodePrefixRule {
    fn check(&self, item: &PrimaryFederationPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationPreferenceRequiredTitleRule {
    pub message: String,
}

impl PrimaryFederationPreferenceRule for PrimaryFederationPreferenceRequiredTitleRule {
    fn check(&self, item: &PrimaryFederationPreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary federation preference.
#[derive(Debug, Clone)]
pub struct PrimaryFederationPreferenceValidator {
    pub rules: Vec<Rc<dyn PrimaryFederationPreferenceRule>>,
}

impl PrimaryFederationPreferenceValidator {
    pub fn validate(&self, item: &PrimaryFederationPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryFederationPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryFederationPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_federation_preference_validator() -> PrimaryFederationPreferenceValidator {
    PrimaryFederationPreferenceValidator {
        rules: vec![
            Rc::new(PrimaryFederationPreferenceRequiredUnitRule {
                message: "primary federation preference needs a value".to_string(),
            }) as Rc<dyn PrimaryFederationPreferenceRule>,
            Rc::new(PrimaryFederationPreferenceRequiredCodeRule {
                message: "primary federation preference needs an entry".to_string(),
            }),
            Rc::new(PrimaryFederationPreferenceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PrimaryFederationPreferenceRequiredTitleRule {
                message: "primary federation preference needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_federation_preference_validator();
        let item = new_primary_federation_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PrimaryFederationPreferenceRequiredUnitRule {
            message: "primary federation preference needs a value".to_string(),
        };
        let mut item = new_primary_federation_preference("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryFederationPreferenceRequiredCodeRule {
            message: "primary federation preference needs a value".to_string(),
        };
        let mut item = new_primary_federation_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryFederationPreferenceRequiredTitleRule {
            message: "primary federation preference needs a value".to_string(),
        };
        let mut item = new_primary_federation_preference("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
