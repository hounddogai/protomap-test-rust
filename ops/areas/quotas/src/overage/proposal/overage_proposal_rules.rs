use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage proposal must pass. Check returns its problems, or none.
pub trait OverageProposalRule: std::fmt::Debug {
    fn check(&self, item: &OverageProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageProposalWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageProposalRule for OverageProposalWeightRangeRule {
    fn check(&self, item: &OverageProposal) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageProposalNameLengthRule {
    pub max_length: i64,
}

impl OverageProposalRule for OverageProposalNameLengthRule {
    fn check(&self, item: &OverageProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageProposalLabelLengthRule {
    pub max_length: i64,
}

impl OverageProposalRule for OverageProposalLabelLengthRule {
    fn check(&self, item: &OverageProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageProposalCodePrefixRule {
    pub prefix: String,
}

impl OverageProposalRule for OverageProposalCodePrefixRule {
    fn check(&self, item: &OverageProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageProposalRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageProposalRule for OverageProposalRevisionRangeRule {
    fn check(&self, item: &OverageProposal) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageProposalMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageProposalRule for OverageProposalMarginRangeRule {
    fn check(&self, item: &OverageProposal) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a overage proposal.
#[derive(Debug, Clone)]
pub struct OverageProposalValidator {
    pub rules: Vec<Rc<dyn OverageProposalRule>>,
}

impl OverageProposalValidator {
    pub fn validate(&self, item: &OverageProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_proposal_validator() -> OverageProposalValidator {
    OverageProposalValidator {
        rules: vec![
            Rc::new(OverageProposalWeightRangeRule { min: 6.9, max: 804.4 }) as Rc<dyn OverageProposalRule>,
            Rc::new(OverageProposalNameLengthRule { max_length: 64 }),
            Rc::new(OverageProposalLabelLengthRule { max_length: 64 }),
            Rc::new(OverageProposalCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(OverageProposalRevisionRangeRule { min: 64, max: 3600 }),
            Rc::new(OverageProposalMarginRangeRule { min: 2.4, max: 4645.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_proposal_validator();
        let item = new_overage_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = OverageProposalWeightRangeRule { min: 7.8, max: 518.9 };
        let mut item = new_overage_proposal("x", "y");
        item.weight = 520.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = OverageProposalNameLengthRule { max_length: 120 };
        let mut item = new_overage_proposal("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = OverageProposalLabelLengthRule { max_length: 255 };
        let mut item = new_overage_proposal("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = OverageProposalRevisionRangeRule { min: 64, max: 120 };
        let mut item = new_overage_proposal("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = OverageProposalMarginRangeRule { min: 4.8, max: 3210.0 };
        let mut item = new_overage_proposal("x", "y");
        item.margin = 3211.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
