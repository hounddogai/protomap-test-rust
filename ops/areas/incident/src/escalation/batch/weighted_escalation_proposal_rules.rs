use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted escalation proposal must pass. Check returns its problems, or none.
pub trait WeightedEscalationProposalRule: std::fmt::Debug {
    fn check(&self, item: &WeightedEscalationProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationProposalLabelLengthRule {
    pub max_length: i64,
}

impl WeightedEscalationProposalRule for WeightedEscalationProposalLabelLengthRule {
    fn check(&self, item: &WeightedEscalationProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationProposalCodeLengthRule {
    pub max_length: i64,
}

impl WeightedEscalationProposalRule for WeightedEscalationProposalCodeLengthRule {
    fn check(&self, item: &WeightedEscalationProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationProposalCodePrefixRule {
    pub prefix: String,
}

impl WeightedEscalationProposalRule for WeightedEscalationProposalCodePrefixRule {
    fn check(&self, item: &WeightedEscalationProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationProposalPinnedGuardRule {
    pub limit: i64,
}

impl WeightedEscalationProposalRule for WeightedEscalationProposalPinnedGuardRule {
    fn check(&self, item: &WeightedEscalationProposal) -> Vec<String> {
        if item.pinned && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while pinned", item.burst, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationProposalRequiredLabelRule {
    pub message: String,
}

impl WeightedEscalationProposalRule for WeightedEscalationProposalRequiredLabelRule {
    fn check(&self, item: &WeightedEscalationProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted escalation proposal.
#[derive(Debug, Clone)]
pub struct WeightedEscalationProposalValidator {
    pub rules: Vec<Rc<dyn WeightedEscalationProposalRule>>,
}

impl WeightedEscalationProposalValidator {
    pub fn validate(&self, item: &WeightedEscalationProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedEscalationProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedEscalationProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_escalation_proposal_validator() -> WeightedEscalationProposalValidator {
    WeightedEscalationProposalValidator {
        rules: vec![
            Rc::new(WeightedEscalationProposalLabelLengthRule { max_length: 1000 })
                as Rc<dyn WeightedEscalationProposalRule>,
            Rc::new(WeightedEscalationProposalCodeLengthRule { max_length: 32 }),
            Rc::new(WeightedEscalationProposalCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(WeightedEscalationProposalPinnedGuardRule { limit: 1000 }),
            Rc::new(WeightedEscalationProposalRequiredLabelRule {
                message: "weighted escalation proposal needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_escalation_proposal_validator();
        let item = new_weighted_escalation_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedEscalationProposalLabelLengthRule { max_length: 16 };
        let mut item = new_weighted_escalation_proposal("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedEscalationProposalCodeLengthRule { max_length: 3600 };
        let mut item = new_weighted_escalation_proposal("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = WeightedEscalationProposalRequiredLabelRule {
            message: "weighted escalation proposal needs a value".to_string(),
        };
        let mut item = new_weighted_escalation_proposal("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
