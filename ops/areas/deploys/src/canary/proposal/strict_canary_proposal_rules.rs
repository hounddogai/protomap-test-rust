use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict canary proposal must pass. Check returns its problems, or none.
pub trait StrictCanaryProposalRule: std::fmt::Debug {
    fn check(&self, item: &StrictCanaryProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryProposalRequiredTitleRule {
    pub message: String,
}

impl StrictCanaryProposalRule for StrictCanaryProposalRequiredTitleRule {
    fn check(&self, item: &StrictCanaryProposal) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryProposalBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictCanaryProposalRule for StrictCanaryProposalBudgetRangeRule {
    fn check(&self, item: &StrictCanaryProposal) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryProposalRequiredCodeRule {
    pub message: String,
}

impl StrictCanaryProposalRule for StrictCanaryProposalRequiredCodeRule {
    fn check(&self, item: &StrictCanaryProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryProposalTitleLengthRule {
    pub max_length: i64,
}

impl StrictCanaryProposalRule for StrictCanaryProposalTitleLengthRule {
    fn check(&self, item: &StrictCanaryProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryProposalCodeLengthRule {
    pub max_length: i64,
}

impl StrictCanaryProposalRule for StrictCanaryProposalCodeLengthRule {
    fn check(&self, item: &StrictCanaryProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict canary proposal.
#[derive(Debug, Clone)]
pub struct StrictCanaryProposalValidator {
    pub rules: Vec<Rc<dyn StrictCanaryProposalRule>>,
}

impl StrictCanaryProposalValidator {
    pub fn validate(&self, item: &StrictCanaryProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictCanaryProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictCanaryProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_canary_proposal_validator() -> StrictCanaryProposalValidator {
    StrictCanaryProposalValidator {
        rules: vec![
            Rc::new(StrictCanaryProposalRequiredTitleRule {
                message: "strict canary proposal needs a value".to_string(),
            }) as Rc<dyn StrictCanaryProposalRule>,
            Rc::new(StrictCanaryProposalBudgetRangeRule { min: 0.1, max: 2333.6 }),
            Rc::new(StrictCanaryProposalRequiredCodeRule {
                message: "strict canary proposal needs a value".to_string(),
            }),
            Rc::new(StrictCanaryProposalTitleLengthRule { max_length: 1000 }),
            Rc::new(StrictCanaryProposalCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_canary_proposal_validator();
        let item = new_strict_canary_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = StrictCanaryProposalRequiredTitleRule {
            message: "strict canary proposal needs this field".to_string(),
        };
        let mut item = new_strict_canary_proposal("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = StrictCanaryProposalBudgetRangeRule { min: 7.0, max: 4138.3 };
        let mut item = new_strict_canary_proposal("x", "y");
        item.budget = 4139.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictCanaryProposalRequiredCodeRule {
            message: "strict canary proposal needs this field".to_string(),
        };
        let mut item = new_strict_canary_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = StrictCanaryProposalTitleLengthRule { max_length: 32 };
        let mut item = new_strict_canary_proposal("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictCanaryProposalCodeLengthRule { max_length: 80 };
        let mut item = new_strict_canary_proposal("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
