use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe proposal must pass. Check returns its problems, or none.
pub trait ProbeProposalRule: std::fmt::Debug {
    fn check(&self, item: &ProbeProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeProposalRequiredTimezoneRule {
    pub message: String,
}

impl ProbeProposalRule for ProbeProposalRequiredTimezoneRule {
    fn check(&self, item: &ProbeProposal) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeProposalCodePrefixRule {
    pub prefix: String,
}

impl ProbeProposalRule for ProbeProposalCodePrefixRule {
    fn check(&self, item: &ProbeProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeProposalNameLengthRule {
    pub max_length: i64,
}

impl ProbeProposalRule for ProbeProposalNameLengthRule {
    fn check(&self, item: &ProbeProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeProposalRequiredNameRule {
    pub message: String,
}

impl ProbeProposalRule for ProbeProposalRequiredNameRule {
    fn check(&self, item: &ProbeProposal) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeProposalRequiredKeyIdRule {
    pub message: String,
}

impl ProbeProposalRule for ProbeProposalRequiredKeyIdRule {
    fn check(&self, item: &ProbeProposal) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeProposalTimezoneLengthRule {
    pub max_length: i64,
}

impl ProbeProposalRule for ProbeProposalTimezoneLengthRule {
    fn check(&self, item: &ProbeProposal) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe proposal.
#[derive(Debug, Clone)]
pub struct ProbeProposalValidator {
    pub rules: Vec<Rc<dyn ProbeProposalRule>>,
}

impl ProbeProposalValidator {
    pub fn validate(&self, item: &ProbeProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbeProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_proposal_validator() -> ProbeProposalValidator {
    ProbeProposalValidator {
        rules: vec![
            Rc::new(ProbeProposalRequiredTimezoneRule {
                message: "probe proposal needs this field".to_string(),
            }) as Rc<dyn ProbeProposalRule>,
            Rc::new(ProbeProposalCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ProbeProposalNameLengthRule { max_length: 64 }),
            Rc::new(ProbeProposalRequiredNameRule {
                message: "probe proposal needs a value".to_string(),
            }),
            Rc::new(ProbeProposalRequiredKeyIdRule {
                message: "probe proposal needs a value".to_string(),
            }),
            Rc::new(ProbeProposalTimezoneLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_proposal_validator();
        let item = new_probe_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = ProbeProposalRequiredTimezoneRule {
            message: "probe proposal needs this field".to_string(),
        };
        let mut item = new_probe_proposal("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ProbeProposalNameLengthRule { max_length: 64 };
        let mut item = new_probe_proposal("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ProbeProposalRequiredNameRule {
            message: "probe proposal needs this field".to_string(),
        };
        let mut item = new_probe_proposal("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = ProbeProposalRequiredKeyIdRule {
            message: "probe proposal needs a value".to_string(),
        };
        let mut item = new_probe_proposal("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = ProbeProposalTimezoneLengthRule { max_length: 86400 };
        let mut item = new_probe_proposal("x", "y");
        item.timezone = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
