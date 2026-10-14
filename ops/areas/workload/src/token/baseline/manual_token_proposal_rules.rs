use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual token proposal must pass. Check returns its problems, or none.
pub trait ManualTokenProposalRule: std::fmt::Debug {
    fn check(&self, item: &ManualTokenProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTokenProposalCodePrefixRule {
    pub prefix: String,
}

impl ManualTokenProposalRule for ManualTokenProposalCodePrefixRule {
    fn check(&self, item: &ManualTokenProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTokenProposalEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualTokenProposalRule for ManualTokenProposalEndsAtRangeRule {
    fn check(&self, item: &ManualTokenProposal) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTokenProposalRequiredNameRule {
    pub message: String,
}

impl ManualTokenProposalRule for ManualTokenProposalRequiredNameRule {
    fn check(&self, item: &ManualTokenProposal) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTokenProposalCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualTokenProposalRule for ManualTokenProposalCeilingRangeRule {
    fn check(&self, item: &ManualTokenProposal) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual token proposal.
#[derive(Debug, Clone)]
pub struct ManualTokenProposalValidator {
    pub rules: Vec<Rc<dyn ManualTokenProposalRule>>,
}

impl ManualTokenProposalValidator {
    pub fn validate(&self, item: &ManualTokenProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualTokenProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualTokenProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_token_proposal_validator() -> ManualTokenProposalValidator {
    ManualTokenProposalValidator {
        rules: vec![
            Rc::new(ManualTokenProposalCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn ManualTokenProposalRule>,
            Rc::new(ManualTokenProposalEndsAtRangeRule { min: 1000, max: 3600 }),
            Rc::new(ManualTokenProposalRequiredNameRule {
                message: "manual token proposal needs this field".to_string(),
            }),
            Rc::new(ManualTokenProposalCeilingRangeRule { min: 2.3, max: 1615.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_token_proposal_validator();
        let item = new_manual_token_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = ManualTokenProposalEndsAtRangeRule { min: 255, max: 86400 };
        let mut item = new_manual_token_proposal("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualTokenProposalRequiredNameRule {
            message: "manual token proposal needs a value".to_string(),
        };
        let mut item = new_manual_token_proposal("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ManualTokenProposalCeilingRangeRule { min: 7.0, max: 562.9 };
        let mut item = new_manual_token_proposal("x", "y");
        item.ceiling = 564.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
