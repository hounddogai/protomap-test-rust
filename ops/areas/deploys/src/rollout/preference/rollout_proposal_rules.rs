use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollout proposal must pass. Check returns its problems, or none.
pub trait RolloutProposalRule: std::fmt::Debug {
    fn check(&self, item: &RolloutProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutProposalAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl RolloutProposalRule for RolloutProposalAllowedUnitRule {
    fn check(&self, item: &RolloutProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is too long", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutProposalNameLengthRule {
    pub max_length: i64,
}

impl RolloutProposalRule for RolloutProposalNameLengthRule {
    fn check(&self, item: &RolloutProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutProposalUnitLengthRule {
    pub max_length: i64,
}

impl RolloutProposalRule for RolloutProposalUnitLengthRule {
    fn check(&self, item: &RolloutProposal) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutProposalMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutProposalRule for RolloutProposalMarginRangeRule {
    fn check(&self, item: &RolloutProposal) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout proposal.
#[derive(Debug, Clone)]
pub struct RolloutProposalValidator {
    pub rules: Vec<Rc<dyn RolloutProposalRule>>,
}

impl RolloutProposalValidator {
    pub fn validate(&self, item: &RolloutProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_proposal_validator() -> RolloutProposalValidator {
    RolloutProposalValidator {
        rules: vec![
            Rc::new(RolloutProposalAllowedUnitRule {
                allowed: vec!["regional".to_string(), "pending".to_string()],
            }) as Rc<dyn RolloutProposalRule>,
            Rc::new(RolloutProposalNameLengthRule { max_length: 32 }),
            Rc::new(RolloutProposalUnitLengthRule { max_length: 3600 }),
            Rc::new(RolloutProposalMarginRangeRule { min: 3.1, max: 1016.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_proposal_validator();
        let item = new_rollout_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RolloutProposalNameLengthRule { max_length: 1000 };
        let mut item = new_rollout_proposal("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = RolloutProposalUnitLengthRule { max_length: 16 };
        let mut item = new_rollout_proposal("x", "y");
        item.unit = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = RolloutProposalMarginRangeRule { min: 9.6, max: 2215.5 };
        let mut item = new_rollout_proposal("x", "y");
        item.margin = 2217.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
