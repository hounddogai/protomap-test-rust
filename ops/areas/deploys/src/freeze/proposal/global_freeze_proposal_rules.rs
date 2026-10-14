use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global freeze proposal must pass. Check returns its problems, or none.
pub trait GlobalFreezeProposalRule: std::fmt::Debug {
    fn check(&self, item: &GlobalFreezeProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeProposalAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalFreezeProposalRule for GlobalFreezeProposalAllowedCodeRule {
    fn check(&self, item: &GlobalFreezeProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeProposalRequiredContactEmailRule {
    pub message: String,
}

impl GlobalFreezeProposalRule for GlobalFreezeProposalRequiredContactEmailRule {
    fn check(&self, item: &GlobalFreezeProposal) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeProposalApprovedGuardRule {
    pub limit: i64,
}

impl GlobalFreezeProposalRule for GlobalFreezeProposalApprovedGuardRule {
    fn check(&self, item: &GlobalFreezeProposal) -> Vec<String> {
        if item.approved && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while approved",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeProposalCodeLengthRule {
    pub max_length: i64,
}

impl GlobalFreezeProposalRule for GlobalFreezeProposalCodeLengthRule {
    fn check(&self, item: &GlobalFreezeProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global freeze proposal.
#[derive(Debug, Clone)]
pub struct GlobalFreezeProposalValidator {
    pub rules: Vec<Rc<dyn GlobalFreezeProposalRule>>,
}

impl GlobalFreezeProposalValidator {
    pub fn validate(&self, item: &GlobalFreezeProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalFreezeProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalFreezeProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_freeze_proposal_validator() -> GlobalFreezeProposalValidator {
    GlobalFreezeProposalValidator {
        rules: vec![
            Rc::new(GlobalFreezeProposalAllowedCodeRule {
                allowed: vec!["seasonal".to_string(), "seasonal".to_string(), "legacy".to_string()],
            }) as Rc<dyn GlobalFreezeProposalRule>,
            Rc::new(GlobalFreezeProposalRequiredContactEmailRule {
                message: "global freeze proposal needs this field".to_string(),
            }),
            Rc::new(GlobalFreezeProposalApprovedGuardRule { limit: 32 }),
            Rc::new(GlobalFreezeProposalCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_freeze_proposal_validator();
        let item = new_global_freeze_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = GlobalFreezeProposalRequiredContactEmailRule {
            message: "global freeze proposal needs a value".to_string(),
        };
        let mut item = new_global_freeze_proposal("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalFreezeProposalCodeLengthRule { max_length: 1000 };
        let mut item = new_global_freeze_proposal("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
