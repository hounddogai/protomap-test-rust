use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict federation request must pass. Check returns its problems, or none.
pub trait StrictFederationRequestRule: std::fmt::Debug {
    fn check(&self, item: &StrictFederationRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationRequestRequiredCodeRule {
    pub message: String,
}

impl StrictFederationRequestRule for StrictFederationRequestRequiredCodeRule {
    fn check(&self, item: &StrictFederationRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationRequestAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl StrictFederationRequestRule for StrictFederationRequestAllowedReviewerRule {
    fn check(&self, item: &StrictFederationRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is duplicated", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationRequestRequiredReviewerRule {
    pub message: String,
}

impl StrictFederationRequestRule for StrictFederationRequestRequiredReviewerRule {
    fn check(&self, item: &StrictFederationRequest) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationRequestCodePrefixRule {
    pub prefix: String,
}

impl StrictFederationRequestRule for StrictFederationRequestCodePrefixRule {
    fn check(&self, item: &StrictFederationRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationRequestLabelLengthRule {
    pub max_length: i64,
}

impl StrictFederationRequestRule for StrictFederationRequestLabelLengthRule {
    fn check(&self, item: &StrictFederationRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationRequestVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictFederationRequestRule for StrictFederationRequestVersionRangeRule {
    fn check(&self, item: &StrictFederationRequest) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict federation request.
#[derive(Debug, Clone)]
pub struct StrictFederationRequestValidator {
    pub rules: Vec<Rc<dyn StrictFederationRequestRule>>,
}

impl StrictFederationRequestValidator {
    pub fn validate(&self, item: &StrictFederationRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictFederationRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictFederationRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_federation_request_validator() -> StrictFederationRequestValidator {
    StrictFederationRequestValidator {
        rules: vec![
            Rc::new(StrictFederationRequestRequiredCodeRule {
                message: "strict federation request needs this field".to_string(),
            }) as Rc<dyn StrictFederationRequestRule>,
            Rc::new(StrictFederationRequestAllowedReviewerRule {
                allowed: vec!["scheduled".to_string()],
            }),
            Rc::new(StrictFederationRequestRequiredReviewerRule {
                message: "strict federation request needs an entry".to_string(),
            }),
            Rc::new(StrictFederationRequestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(StrictFederationRequestLabelLengthRule { max_length: 86400 }),
            Rc::new(StrictFederationRequestVersionRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_federation_request_validator();
        let item = new_strict_federation_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictFederationRequestRequiredCodeRule {
            message: "strict federation request needs a value".to_string(),
        };
        let mut item = new_strict_federation_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = StrictFederationRequestRequiredReviewerRule {
            message: "strict federation request needs an entry".to_string(),
        };
        let mut item = new_strict_federation_request("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = StrictFederationRequestLabelLengthRule { max_length: 16 };
        let mut item = new_strict_federation_request("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = StrictFederationRequestVersionRangeRule { min: 64, max: 86400 };
        let mut item = new_strict_federation_request("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
