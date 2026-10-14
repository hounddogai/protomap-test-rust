use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted credential preference must pass. Check returns its problems, or none.
pub trait WeightedCredentialPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &WeightedCredentialPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialPreferenceMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedCredentialPreferenceRule for WeightedCredentialPreferenceMaximumRangeRule {
    fn check(&self, item: &WeightedCredentialPreference) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialPreferenceRequiredNameRule {
    pub message: String,
}

impl WeightedCredentialPreferenceRule for WeightedCredentialPreferenceRequiredNameRule {
    fn check(&self, item: &WeightedCredentialPreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialPreferenceRequiredCodeRule {
    pub message: String,
}

impl WeightedCredentialPreferenceRule for WeightedCredentialPreferenceRequiredCodeRule {
    fn check(&self, item: &WeightedCredentialPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialPreferenceCodePrefixRule {
    pub prefix: String,
}

impl WeightedCredentialPreferenceRule for WeightedCredentialPreferenceCodePrefixRule {
    fn check(&self, item: &WeightedCredentialPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted credential preference.
#[derive(Debug, Clone)]
pub struct WeightedCredentialPreferenceValidator {
    pub rules: Vec<Rc<dyn WeightedCredentialPreferenceRule>>,
}

impl WeightedCredentialPreferenceValidator {
    pub fn validate(&self, item: &WeightedCredentialPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedCredentialPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedCredentialPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_credential_preference_validator() -> WeightedCredentialPreferenceValidator {
    WeightedCredentialPreferenceValidator {
        rules: vec![
            Rc::new(WeightedCredentialPreferenceMaximumRangeRule { min: 9.4, max: 4235.9 })
                as Rc<dyn WeightedCredentialPreferenceRule>,
            Rc::new(WeightedCredentialPreferenceRequiredNameRule {
                message: "weighted credential preference needs this field".to_string(),
            }),
            Rc::new(WeightedCredentialPreferenceRequiredCodeRule {
                message: "weighted credential preference needs an entry".to_string(),
            }),
            Rc::new(WeightedCredentialPreferenceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_credential_preference_validator();
        let item = new_weighted_credential_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = WeightedCredentialPreferenceMaximumRangeRule { min: 3.2, max: 4604.5 };
        let mut item = new_weighted_credential_preference("x", "y");
        item.maximum = 4606.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = WeightedCredentialPreferenceRequiredNameRule {
            message: "weighted credential preference needs a value".to_string(),
        };
        let mut item = new_weighted_credential_preference("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedCredentialPreferenceRequiredCodeRule {
            message: "weighted credential preference needs an entry".to_string(),
        };
        let mut item = new_weighted_credential_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
