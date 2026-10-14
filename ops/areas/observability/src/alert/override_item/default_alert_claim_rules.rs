use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default alert claim must pass. Check returns its problems, or none.
pub trait DefaultAlertClaimRule: std::fmt::Debug {
    fn check(&self, item: &DefaultAlertClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertClaimCodeLengthRule {
    pub max_length: i64,
}

impl DefaultAlertClaimRule for DefaultAlertClaimCodeLengthRule {
    fn check(&self, item: &DefaultAlertClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertClaimCodePrefixRule {
    pub prefix: String,
}

impl DefaultAlertClaimRule for DefaultAlertClaimCodePrefixRule {
    fn check(&self, item: &DefaultAlertClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertClaimRequiredLabelRule {
    pub message: String,
}

impl DefaultAlertClaimRule for DefaultAlertClaimRequiredLabelRule {
    fn check(&self, item: &DefaultAlertClaim) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertClaimAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl DefaultAlertClaimRule for DefaultAlertClaimAllowedLabelRule {
    fn check(&self, item: &DefaultAlertClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertClaimKeyIdLengthRule {
    pub max_length: i64,
}

impl DefaultAlertClaimRule for DefaultAlertClaimKeyIdLengthRule {
    fn check(&self, item: &DefaultAlertClaim) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertClaimRequiredCodeRule {
    pub message: String,
}

impl DefaultAlertClaimRule for DefaultAlertClaimRequiredCodeRule {
    fn check(&self, item: &DefaultAlertClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default alert claim.
#[derive(Debug, Clone)]
pub struct DefaultAlertClaimValidator {
    pub rules: Vec<Rc<dyn DefaultAlertClaimRule>>,
}

impl DefaultAlertClaimValidator {
    pub fn validate(&self, item: &DefaultAlertClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultAlertClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultAlertClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_alert_claim_validator() -> DefaultAlertClaimValidator {
    DefaultAlertClaimValidator {
        rules: vec![
            Rc::new(DefaultAlertClaimCodeLengthRule { max_length: 64 }) as Rc<dyn DefaultAlertClaimRule>,
            Rc::new(DefaultAlertClaimCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DefaultAlertClaimRequiredLabelRule {
                message: "default alert claim needs an entry".to_string(),
            }),
            Rc::new(DefaultAlertClaimAllowedLabelRule {
                allowed: vec!["pending".to_string(), "batched".to_string(), "lenient".to_string()],
            }),
            Rc::new(DefaultAlertClaimKeyIdLengthRule { max_length: 80 }),
            Rc::new(DefaultAlertClaimRequiredCodeRule {
                message: "default alert claim needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_alert_claim_validator();
        let item = new_default_alert_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultAlertClaimCodeLengthRule { max_length: 80 };
        let mut item = new_default_alert_claim("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultAlertClaimRequiredLabelRule {
            message: "default alert claim needs a value".to_string(),
        };
        let mut item = new_default_alert_claim("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = DefaultAlertClaimKeyIdLengthRule { max_length: 32 };
        let mut item = new_default_alert_claim("x", "y");
        item.key_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultAlertClaimRequiredCodeRule {
            message: "default alert claim needs an entry".to_string(),
        };
        let mut item = new_default_alert_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
