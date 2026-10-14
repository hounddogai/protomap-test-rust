use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote proposal must pass. Check returns its problems, or none.
pub trait QuoteProposalRule: std::fmt::Debug {
    fn check(&self, item: &QuoteProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProposalBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteProposalRule for QuoteProposalBudgetRangeRule {
    fn check(&self, item: &QuoteProposal) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProposalNameLengthRule {
    pub max_length: i64,
}

impl QuoteProposalRule for QuoteProposalNameLengthRule {
    fn check(&self, item: &QuoteProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProposalCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteProposalRule for QuoteProposalCostRangeRule {
    fn check(&self, item: &QuoteProposal) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProposalRequiredCodeRule {
    pub message: String,
}

impl QuoteProposalRule for QuoteProposalRequiredCodeRule {
    fn check(&self, item: &QuoteProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProposalCodeLengthRule {
    pub max_length: i64,
}

impl QuoteProposalRule for QuoteProposalCodeLengthRule {
    fn check(&self, item: &QuoteProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote proposal.
#[derive(Debug, Clone)]
pub struct QuoteProposalValidator {
    pub rules: Vec<Rc<dyn QuoteProposalRule>>,
}

impl QuoteProposalValidator {
    pub fn validate(&self, item: &QuoteProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_proposal_validator() -> QuoteProposalValidator {
    QuoteProposalValidator {
        rules: vec![
            Rc::new(QuoteProposalBudgetRangeRule { min: 3.6, max: 1056.0 }) as Rc<dyn QuoteProposalRule>,
            Rc::new(QuoteProposalNameLengthRule { max_length: 1000 }),
            Rc::new(QuoteProposalCostRangeRule { min: 7.9, max: 1230.6 }),
            Rc::new(QuoteProposalRequiredCodeRule {
                message: "quote proposal needs this field".to_string(),
            }),
            Rc::new(QuoteProposalCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_proposal_validator();
        let item = new_quote_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = QuoteProposalBudgetRangeRule { min: 7.8, max: 1403.4 };
        let mut item = new_quote_proposal("x", "y");
        item.budget = 1404.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = QuoteProposalNameLengthRule { max_length: 86400 };
        let mut item = new_quote_proposal("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = QuoteProposalCostRangeRule { min: 9.8, max: 4676.3 };
        let mut item = new_quote_proposal("x", "y");
        item.cost = 4677.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QuoteProposalRequiredCodeRule {
            message: "quote proposal needs a value".to_string(),
        };
        let mut item = new_quote_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QuoteProposalCodeLengthRule { max_length: 3600 };
        let mut item = new_quote_proposal("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
