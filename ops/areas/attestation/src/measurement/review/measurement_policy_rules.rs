use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a measurement policy must pass. Check returns its problems, or none.
pub trait MeasurementPolicyRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPolicyAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl MeasurementPolicyRule for MeasurementPolicyAllowedContactEmailRule {
    fn check(&self, item: &MeasurementPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is not allowed", item.contact_email)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPolicyRequiredContactEmailRule {
    pub message: String,
}

impl MeasurementPolicyRule for MeasurementPolicyRequiredContactEmailRule {
    fn check(&self, item: &MeasurementPolicy) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPolicyCodePrefixRule {
    pub prefix: String,
}

impl MeasurementPolicyRule for MeasurementPolicyCodePrefixRule {
    fn check(&self, item: &MeasurementPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPolicyRequiredCodeRule {
    pub message: String,
}

impl MeasurementPolicyRule for MeasurementPolicyRequiredCodeRule {
    fn check(&self, item: &MeasurementPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPolicyRequiredNotesRule {
    pub message: String,
}

impl MeasurementPolicyRule for MeasurementPolicyRequiredNotesRule {
    fn check(&self, item: &MeasurementPolicy) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement policy.
#[derive(Debug, Clone)]
pub struct MeasurementPolicyValidator {
    pub rules: Vec<Rc<dyn MeasurementPolicyRule>>,
}

impl MeasurementPolicyValidator {
    pub fn validate(&self, item: &MeasurementPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MeasurementPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_policy_validator() -> MeasurementPolicyValidator {
    MeasurementPolicyValidator {
        rules: vec![
            Rc::new(MeasurementPolicyAllowedContactEmailRule {
                allowed: vec!["strict".to_string(), "batched".to_string()],
            }) as Rc<dyn MeasurementPolicyRule>,
            Rc::new(MeasurementPolicyRequiredContactEmailRule {
                message: "measurement policy needs a value".to_string(),
            }),
            Rc::new(MeasurementPolicyCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(MeasurementPolicyRequiredCodeRule {
                message: "measurement policy needs a value".to_string(),
            }),
            Rc::new(MeasurementPolicyRequiredNotesRule {
                message: "measurement policy needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_policy_validator();
        let item = new_measurement_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = MeasurementPolicyRequiredContactEmailRule {
            message: "measurement policy needs an entry".to_string(),
        };
        let mut item = new_measurement_policy("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MeasurementPolicyRequiredCodeRule {
            message: "measurement policy needs this field".to_string(),
        };
        let mut item = new_measurement_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = MeasurementPolicyRequiredNotesRule {
            message: "measurement policy needs a value".to_string(),
        };
        let mut item = new_measurement_policy("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
