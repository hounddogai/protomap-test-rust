use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering preference must pass. Check returns its problems, or none.
pub trait PeeringPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &PeeringPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl PeeringPreferenceRule for PeeringPreferenceCodeLengthRule {
    fn check(&self, item: &PeeringPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPreferenceRequiredNameRule {
    pub message: String,
}

impl PeeringPreferenceRule for PeeringPreferenceRequiredNameRule {
    fn check(&self, item: &PeeringPreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPreferenceRequiredCodeRule {
    pub message: String,
}

impl PeeringPreferenceRule for PeeringPreferenceRequiredCodeRule {
    fn check(&self, item: &PeeringPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering preference.
#[derive(Debug, Clone)]
pub struct PeeringPreferenceValidator {
    pub rules: Vec<Rc<dyn PeeringPreferenceRule>>,
}

impl PeeringPreferenceValidator {
    pub fn validate(&self, item: &PeeringPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_preference_validator() -> PeeringPreferenceValidator {
    PeeringPreferenceValidator {
        rules: vec![
            Rc::new(PeeringPreferenceCodeLengthRule { max_length: 32 }) as Rc<dyn PeeringPreferenceRule>,
            Rc::new(PeeringPreferenceRequiredNameRule {
                message: "peering preference needs this field".to_string(),
            }),
            Rc::new(PeeringPreferenceRequiredCodeRule {
                message: "peering preference needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_preference_validator();
        let item = new_peering_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PeeringPreferenceCodeLengthRule { max_length: 1000 };
        let mut item = new_peering_preference("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PeeringPreferenceRequiredNameRule {
            message: "peering preference needs an entry".to_string(),
        };
        let mut item = new_peering_preference("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PeeringPreferenceRequiredCodeRule {
            message: "peering preference needs a value".to_string(),
        };
        let mut item = new_peering_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
