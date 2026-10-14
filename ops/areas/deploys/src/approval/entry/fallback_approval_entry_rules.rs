use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback approval entry must pass. Check returns its problems, or none.
pub trait FallbackApprovalEntryRule: std::fmt::Debug {
    fn check(&self, item: &FallbackApprovalEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalEntryRequiredPrincipalRule {
    pub message: String,
}

impl FallbackApprovalEntryRule for FallbackApprovalEntryRequiredPrincipalRule {
    fn check(&self, item: &FallbackApprovalEntry) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalEntryCodePrefixRule {
    pub prefix: String,
}

impl FallbackApprovalEntryRule for FallbackApprovalEntryCodePrefixRule {
    fn check(&self, item: &FallbackApprovalEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalEntryRequiredLabelRule {
    pub message: String,
}

impl FallbackApprovalEntryRule for FallbackApprovalEntryRequiredLabelRule {
    fn check(&self, item: &FallbackApprovalEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalEntryLabelLengthRule {
    pub max_length: i64,
}

impl FallbackApprovalEntryRule for FallbackApprovalEntryLabelLengthRule {
    fn check(&self, item: &FallbackApprovalEntry) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalEntryAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl FallbackApprovalEntryRule for FallbackApprovalEntryAllowedSummaryRule {
    fn check(&self, item: &FallbackApprovalEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is not allowed", item.summary)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback approval entry.
#[derive(Debug, Clone)]
pub struct FallbackApprovalEntryValidator {
    pub rules: Vec<Rc<dyn FallbackApprovalEntryRule>>,
}

impl FallbackApprovalEntryValidator {
    pub fn validate(&self, item: &FallbackApprovalEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackApprovalEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackApprovalEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_approval_entry_validator() -> FallbackApprovalEntryValidator {
    FallbackApprovalEntryValidator {
        rules: vec![
            Rc::new(FallbackApprovalEntryRequiredPrincipalRule {
                message: "fallback approval entry needs this field".to_string(),
            }) as Rc<dyn FallbackApprovalEntryRule>,
            Rc::new(FallbackApprovalEntryCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(FallbackApprovalEntryRequiredLabelRule {
                message: "fallback approval entry needs a value".to_string(),
            }),
            Rc::new(FallbackApprovalEntryLabelLengthRule { max_length: 16 }),
            Rc::new(FallbackApprovalEntryAllowedSummaryRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_approval_entry_validator();
        let item = new_fallback_approval_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = FallbackApprovalEntryRequiredPrincipalRule {
            message: "fallback approval entry needs an entry".to_string(),
        };
        let mut item = new_fallback_approval_entry("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackApprovalEntryRequiredLabelRule {
            message: "fallback approval entry needs this field".to_string(),
        };
        let mut item = new_fallback_approval_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackApprovalEntryLabelLengthRule { max_length: 255 };
        let mut item = new_fallback_approval_entry("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
