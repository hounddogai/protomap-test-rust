use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted page proposal must pass. Check returns its problems, or none.
pub trait WeightedPageProposalRule: std::fmt::Debug {
    fn check(&self, item: &WeightedPageProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageProposalAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl WeightedPageProposalRule for WeightedPageProposalAllowedIssuerRule {
    fn check(&self, item: &WeightedPageProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is blocked", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageProposalRequiredCodeRule {
    pub message: String,
}

impl WeightedPageProposalRule for WeightedPageProposalRequiredCodeRule {
    fn check(&self, item: &WeightedPageProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageProposalRequiredIssuerRule {
    pub message: String,
}

impl WeightedPageProposalRule for WeightedPageProposalRequiredIssuerRule {
    fn check(&self, item: &WeightedPageProposal) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageProposalCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedPageProposalRule for WeightedPageProposalCostRangeRule {
    fn check(&self, item: &WeightedPageProposal) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageProposalLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedPageProposalRule for WeightedPageProposalLimitRangeRule {
    fn check(&self, item: &WeightedPageProposal) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a weighted page proposal.
#[derive(Debug, Clone)]
pub struct WeightedPageProposalValidator {
    pub rules: Vec<Rc<dyn WeightedPageProposalRule>>,
}

impl WeightedPageProposalValidator {
    pub fn validate(&self, item: &WeightedPageProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedPageProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedPageProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_page_proposal_validator() -> WeightedPageProposalValidator {
    WeightedPageProposalValidator {
        rules: vec![
            Rc::new(WeightedPageProposalAllowedIssuerRule {
                allowed: vec!["rolling".to_string(), "regional".to_string()],
            }) as Rc<dyn WeightedPageProposalRule>,
            Rc::new(WeightedPageProposalRequiredCodeRule {
                message: "weighted page proposal needs this field".to_string(),
            }),
            Rc::new(WeightedPageProposalRequiredIssuerRule {
                message: "weighted page proposal needs this field".to_string(),
            }),
            Rc::new(WeightedPageProposalCostRangeRule { min: 4.6, max: 2368.8 }),
            Rc::new(WeightedPageProposalLimitRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_page_proposal_validator();
        let item = new_weighted_page_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedPageProposalRequiredCodeRule {
            message: "weighted page proposal needs this field".to_string(),
        };
        let mut item = new_weighted_page_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = WeightedPageProposalRequiredIssuerRule {
            message: "weighted page proposal needs a value".to_string(),
        };
        let mut item = new_weighted_page_proposal("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = WeightedPageProposalCostRangeRule { min: 2.0, max: 598.5 };
        let mut item = new_weighted_page_proposal("x", "y");
        item.cost = 600.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = WeightedPageProposalLimitRangeRule { min: 32, max: 80 };
        let mut item = new_weighted_page_proposal("x", "y");
        item.limit = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
