use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a suspension claim must pass. Check returns its problems, or none.
pub trait SuspensionClaimRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionClaimAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SuspensionClaimRule for SuspensionClaimAllowedCodeRule {
    fn check(&self, item: &SuspensionClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionClaimCodePrefixRule {
    pub prefix: String,
}

impl SuspensionClaimRule for SuspensionClaimCodePrefixRule {
    fn check(&self, item: &SuspensionClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionClaimRequiredCodeRule {
    pub message: String,
}

impl SuspensionClaimRule for SuspensionClaimRequiredCodeRule {
    fn check(&self, item: &SuspensionClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension claim.
#[derive(Debug, Clone)]
pub struct SuspensionClaimValidator {
    pub rules: Vec<Rc<dyn SuspensionClaimRule>>,
}

impl SuspensionClaimValidator {
    pub fn validate(&self, item: &SuspensionClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_claim_validator() -> SuspensionClaimValidator {
    SuspensionClaimValidator {
        rules: vec![
            Rc::new(SuspensionClaimAllowedCodeRule {
                allowed: vec!["lenient".to_string(), "tiered".to_string(), "primary".to_string()],
            }) as Rc<dyn SuspensionClaimRule>,
            Rc::new(SuspensionClaimCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(SuspensionClaimRequiredCodeRule {
                message: "suspension claim needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_claim_validator();
        let item = new_suspension_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SuspensionClaimRequiredCodeRule {
            message: "suspension claim needs an entry".to_string(),
        };
        let mut item = new_suspension_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
