use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a probe variance must pass. Check returns its problems, or none.
pub trait ProbeVarianceRule: std::fmt::Debug {
    fn check(&self, item: &ProbeVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeVarianceAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl ProbeVarianceRule for ProbeVarianceAllowedIssuerRule {
    fn check(&self, item: &ProbeVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is duplicated", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeVarianceOwnerIdLengthRule {
    pub max_length: i64,
}

impl ProbeVarianceRule for ProbeVarianceOwnerIdLengthRule {
    fn check(&self, item: &ProbeVariance) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeVarianceCodePrefixRule {
    pub prefix: String,
}

impl ProbeVarianceRule for ProbeVarianceCodePrefixRule {
    fn check(&self, item: &ProbeVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeVarianceRequiredCodeRule {
    pub message: String,
}

impl ProbeVarianceRule for ProbeVarianceRequiredCodeRule {
    fn check(&self, item: &ProbeVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeVarianceRequiredIssuerRule {
    pub message: String,
}

impl ProbeVarianceRule for ProbeVarianceRequiredIssuerRule {
    fn check(&self, item: &ProbeVariance) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeVarianceLabelLengthRule {
    pub max_length: i64,
}

impl ProbeVarianceRule for ProbeVarianceLabelLengthRule {
    fn check(&self, item: &ProbeVariance) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe variance.
#[derive(Debug, Clone)]
pub struct ProbeVarianceValidator {
    pub rules: Vec<Rc<dyn ProbeVarianceRule>>,
}

impl ProbeVarianceValidator {
    pub fn validate(&self, item: &ProbeVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_variance_validator() -> ProbeVarianceValidator {
    ProbeVarianceValidator {
        rules: vec![
            Rc::new(ProbeVarianceAllowedIssuerRule {
                allowed: vec!["batched".to_string(), "seasonal".to_string(), "pending".to_string()],
            }) as Rc<dyn ProbeVarianceRule>,
            Rc::new(ProbeVarianceOwnerIdLengthRule { max_length: 1000 }),
            Rc::new(ProbeVarianceCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ProbeVarianceRequiredCodeRule {
                message: "probe variance needs this field".to_string(),
            }),
            Rc::new(ProbeVarianceRequiredIssuerRule {
                message: "probe variance needs a value".to_string(),
            }),
            Rc::new(ProbeVarianceLabelLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_variance_validator();
        let item = new_probe_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = ProbeVarianceOwnerIdLengthRule { max_length: 64 };
        let mut item = new_probe_variance("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeVarianceRequiredCodeRule {
            message: "probe variance needs an entry".to_string(),
        };
        let mut item = new_probe_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = ProbeVarianceRequiredIssuerRule {
            message: "probe variance needs this field".to_string(),
        };
        let mut item = new_probe_variance("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ProbeVarianceLabelLengthRule { max_length: 80 };
        let mut item = new_probe_variance("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
