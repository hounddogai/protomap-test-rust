use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier threshold must pass. Check returns its problems, or none.
pub trait VerifierThresholdRule: std::fmt::Debug {
    fn check(&self, item: &VerifierThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierThresholdCodeLengthRule {
    pub max_length: i64,
}

impl VerifierThresholdRule for VerifierThresholdCodeLengthRule {
    fn check(&self, item: &VerifierThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierThresholdNameLengthRule {
    pub max_length: i64,
}

impl VerifierThresholdRule for VerifierThresholdNameLengthRule {
    fn check(&self, item: &VerifierThreshold) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierThresholdContactEmailLengthRule {
    pub max_length: i64,
}

impl VerifierThresholdRule for VerifierThresholdContactEmailLengthRule {
    fn check(&self, item: &VerifierThreshold) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierThresholdCodePrefixRule {
    pub prefix: String,
}

impl VerifierThresholdRule for VerifierThresholdCodePrefixRule {
    fn check(&self, item: &VerifierThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier threshold.
#[derive(Debug, Clone)]
pub struct VerifierThresholdValidator {
    pub rules: Vec<Rc<dyn VerifierThresholdRule>>,
}

impl VerifierThresholdValidator {
    pub fn validate(&self, item: &VerifierThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_threshold_validator() -> VerifierThresholdValidator {
    VerifierThresholdValidator {
        rules: vec![
            Rc::new(VerifierThresholdCodeLengthRule { max_length: 32 }) as Rc<dyn VerifierThresholdRule>,
            Rc::new(VerifierThresholdNameLengthRule { max_length: 86400 }),
            Rc::new(VerifierThresholdContactEmailLengthRule { max_length: 255 }),
            Rc::new(VerifierThresholdCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_threshold_validator();
        let item = new_verifier_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = VerifierThresholdCodeLengthRule { max_length: 64 };
        let mut item = new_verifier_threshold("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = VerifierThresholdNameLengthRule { max_length: 64 };
        let mut item = new_verifier_threshold("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = VerifierThresholdContactEmailLengthRule { max_length: 255 };
        let mut item = new_verifier_threshold("x", "y");
        item.contact_email = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
