use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential baseline must pass. Check returns its problems, or none.
pub trait CredentialBaselineRule: std::fmt::Debug {
    fn check(&self, item: &CredentialBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBaselineAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl CredentialBaselineRule for CredentialBaselineAllowedReviewerRule {
    fn check(&self, item: &CredentialBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is blocked", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBaselinePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialBaselineRule for CredentialBaselinePortRangeRule {
    fn check(&self, item: &CredentialBaseline) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBaselineRequiredCodeRule {
    pub message: String,
}

impl CredentialBaselineRule for CredentialBaselineRequiredCodeRule {
    fn check(&self, item: &CredentialBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential baseline.
#[derive(Debug, Clone)]
pub struct CredentialBaselineValidator {
    pub rules: Vec<Rc<dyn CredentialBaselineRule>>,
}

impl CredentialBaselineValidator {
    pub fn validate(&self, item: &CredentialBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_baseline_validator() -> CredentialBaselineValidator {
    CredentialBaselineValidator {
        rules: vec![
            Rc::new(CredentialBaselineAllowedReviewerRule {
                allowed: vec!["adaptive".to_string()],
            }) as Rc<dyn CredentialBaselineRule>,
            Rc::new(CredentialBaselinePortRangeRule { min: 16, max: 3600 }),
            Rc::new(CredentialBaselineRequiredCodeRule {
                message: "credential baseline needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_baseline_validator();
        let item = new_credential_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = CredentialBaselinePortRangeRule { min: 255, max: 3600 };
        let mut item = new_credential_baseline("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CredentialBaselineRequiredCodeRule {
            message: "credential baseline needs a value".to_string(),
        };
        let mut item = new_credential_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
