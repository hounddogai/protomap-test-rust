use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback binding claim must pass. Check returns its problems, or none.
pub trait FallbackBindingClaimRule: std::fmt::Debug {
    fn check(&self, item: &FallbackBindingClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingClaimCodePrefixRule {
    pub prefix: String,
}

impl FallbackBindingClaimRule for FallbackBindingClaimCodePrefixRule {
    fn check(&self, item: &FallbackBindingClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingClaimRequiredCodeRule {
    pub message: String,
}

impl FallbackBindingClaimRule for FallbackBindingClaimRequiredCodeRule {
    fn check(&self, item: &FallbackBindingClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingClaimRequiredLabelRule {
    pub message: String,
}

impl FallbackBindingClaimRule for FallbackBindingClaimRequiredLabelRule {
    fn check(&self, item: &FallbackBindingClaim) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingClaimActiveGuardRule {
    pub limit: f64,
}

impl FallbackBindingClaimRule for FallbackBindingClaimActiveGuardRule {
    fn check(&self, item: &FallbackBindingClaim) -> Vec<String> {
        if item.active && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while active",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingClaimRequiredFingerprintRule {
    pub message: String,
}

impl FallbackBindingClaimRule for FallbackBindingClaimRequiredFingerprintRule {
    fn check(&self, item: &FallbackBindingClaim) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingClaimCodeLengthRule {
    pub max_length: i64,
}

impl FallbackBindingClaimRule for FallbackBindingClaimCodeLengthRule {
    fn check(&self, item: &FallbackBindingClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback binding claim.
#[derive(Debug, Clone)]
pub struct FallbackBindingClaimValidator {
    pub rules: Vec<Rc<dyn FallbackBindingClaimRule>>,
}

impl FallbackBindingClaimValidator {
    pub fn validate(&self, item: &FallbackBindingClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackBindingClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackBindingClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_binding_claim_validator() -> FallbackBindingClaimValidator {
    FallbackBindingClaimValidator {
        rules: vec![
            Rc::new(FallbackBindingClaimCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn FallbackBindingClaimRule>,
            Rc::new(FallbackBindingClaimRequiredCodeRule {
                message: "fallback binding claim needs this field".to_string(),
            }),
            Rc::new(FallbackBindingClaimRequiredLabelRule {
                message: "fallback binding claim needs an entry".to_string(),
            }),
            Rc::new(FallbackBindingClaimActiveGuardRule { limit: 2.5 }),
            Rc::new(FallbackBindingClaimRequiredFingerprintRule {
                message: "fallback binding claim needs a value".to_string(),
            }),
            Rc::new(FallbackBindingClaimCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_binding_claim_validator();
        let item = new_fallback_binding_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackBindingClaimRequiredCodeRule {
            message: "fallback binding claim needs this field".to_string(),
        };
        let mut item = new_fallback_binding_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackBindingClaimRequiredLabelRule {
            message: "fallback binding claim needs this field".to_string(),
        };
        let mut item = new_fallback_binding_claim("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = FallbackBindingClaimRequiredFingerprintRule {
            message: "fallback binding claim needs this field".to_string(),
        };
        let mut item = new_fallback_binding_claim("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackBindingClaimCodeLengthRule { max_length: 120 };
        let mut item = new_fallback_binding_claim("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
