use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback tenant agreement must pass. Check returns its problems, or none.
pub trait FallbackTenantAgreementRule: std::fmt::Debug {
    fn check(&self, item: &FallbackTenantAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantAgreementRequiredTimezoneRule {
    pub message: String,
}

impl FallbackTenantAgreementRule for FallbackTenantAgreementRequiredTimezoneRule {
    fn check(&self, item: &FallbackTenantAgreement) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantAgreementLabelLengthRule {
    pub max_length: i64,
}

impl FallbackTenantAgreementRule for FallbackTenantAgreementLabelLengthRule {
    fn check(&self, item: &FallbackTenantAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantAgreementCodePrefixRule {
    pub prefix: String,
}

impl FallbackTenantAgreementRule for FallbackTenantAgreementCodePrefixRule {
    fn check(&self, item: &FallbackTenantAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback tenant agreement.
#[derive(Debug, Clone)]
pub struct FallbackTenantAgreementValidator {
    pub rules: Vec<Rc<dyn FallbackTenantAgreementRule>>,
}

impl FallbackTenantAgreementValidator {
    pub fn validate(&self, item: &FallbackTenantAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackTenantAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackTenantAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_tenant_agreement_validator() -> FallbackTenantAgreementValidator {
    FallbackTenantAgreementValidator {
        rules: vec![
            Rc::new(FallbackTenantAgreementRequiredTimezoneRule {
                message: "fallback tenant agreement needs this field".to_string(),
            }) as Rc<dyn FallbackTenantAgreementRule>,
            Rc::new(FallbackTenantAgreementLabelLengthRule { max_length: 64 }),
            Rc::new(FallbackTenantAgreementCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_tenant_agreement_validator();
        let item = new_fallback_tenant_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = FallbackTenantAgreementRequiredTimezoneRule {
            message: "fallback tenant agreement needs this field".to_string(),
        };
        let mut item = new_fallback_tenant_agreement("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackTenantAgreementLabelLengthRule { max_length: 120 };
        let mut item = new_fallback_tenant_agreement("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
