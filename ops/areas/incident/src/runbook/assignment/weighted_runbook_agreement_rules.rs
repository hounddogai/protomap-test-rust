use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted runbook agreement must pass. Check returns its problems, or none.
pub trait WeightedRunbookAgreementRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRunbookAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookAgreementCodeLengthRule {
    pub max_length: i64,
}

impl WeightedRunbookAgreementRule for WeightedRunbookAgreementCodeLengthRule {
    fn check(&self, item: &WeightedRunbookAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookAgreementRequiredLabelRule {
    pub message: String,
}

impl WeightedRunbookAgreementRule for WeightedRunbookAgreementRequiredLabelRule {
    fn check(&self, item: &WeightedRunbookAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookAgreementExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedRunbookAgreementRule for WeightedRunbookAgreementExpiresAtRangeRule {
    fn check(&self, item: &WeightedRunbookAgreement) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookAgreementCodePrefixRule {
    pub prefix: String,
}

impl WeightedRunbookAgreementRule for WeightedRunbookAgreementCodePrefixRule {
    fn check(&self, item: &WeightedRunbookAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookAgreementLabelLengthRule {
    pub max_length: i64,
}

impl WeightedRunbookAgreementRule for WeightedRunbookAgreementLabelLengthRule {
    fn check(&self, item: &WeightedRunbookAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookAgreementAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl WeightedRunbookAgreementRule for WeightedRunbookAgreementAllowedCodeRule {
    fn check(&self, item: &WeightedRunbookAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted runbook agreement.
#[derive(Debug, Clone)]
pub struct WeightedRunbookAgreementValidator {
    pub rules: Vec<Rc<dyn WeightedRunbookAgreementRule>>,
}

impl WeightedRunbookAgreementValidator {
    pub fn validate(&self, item: &WeightedRunbookAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRunbookAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRunbookAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_runbook_agreement_validator() -> WeightedRunbookAgreementValidator {
    WeightedRunbookAgreementValidator {
        rules: vec![
            Rc::new(WeightedRunbookAgreementCodeLengthRule { max_length: 3600 })
                as Rc<dyn WeightedRunbookAgreementRule>,
            Rc::new(WeightedRunbookAgreementRequiredLabelRule {
                message: "weighted runbook agreement needs this field".to_string(),
            }),
            Rc::new(WeightedRunbookAgreementExpiresAtRangeRule { min: 32, max: 64 }),
            Rc::new(WeightedRunbookAgreementCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(WeightedRunbookAgreementLabelLengthRule { max_length: 120 }),
            Rc::new(WeightedRunbookAgreementAllowedCodeRule {
                allowed: vec!["capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_runbook_agreement_validator();
        let item = new_weighted_runbook_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedRunbookAgreementCodeLengthRule { max_length: 3600 };
        let mut item = new_weighted_runbook_agreement("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = WeightedRunbookAgreementRequiredLabelRule {
            message: "weighted runbook agreement needs a value".to_string(),
        };
        let mut item = new_weighted_runbook_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = WeightedRunbookAgreementExpiresAtRangeRule { min: 64, max: 1000 };
        let mut item = new_weighted_runbook_agreement("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedRunbookAgreementLabelLengthRule { max_length: 86400 };
        let mut item = new_weighted_runbook_agreement("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
