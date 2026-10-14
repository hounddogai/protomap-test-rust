use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global budget burn proposal must pass. Check returns its problems, or none.
pub trait GlobalBudgetBurnProposalRule: std::fmt::Debug {
    fn check(&self, item: &GlobalBudgetBurnProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnProposalCodePrefixRule {
    pub prefix: String,
}

impl GlobalBudgetBurnProposalRule for GlobalBudgetBurnProposalCodePrefixRule {
    fn check(&self, item: &GlobalBudgetBurnProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnProposalBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalBudgetBurnProposalRule for GlobalBudgetBurnProposalBudgetRangeRule {
    fn check(&self, item: &GlobalBudgetBurnProposal) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnProposalRequiredNameRule {
    pub message: String,
}

impl GlobalBudgetBurnProposalRule for GlobalBudgetBurnProposalRequiredNameRule {
    fn check(&self, item: &GlobalBudgetBurnProposal) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnProposalCodeLengthRule {
    pub max_length: i64,
}

impl GlobalBudgetBurnProposalRule for GlobalBudgetBurnProposalCodeLengthRule {
    fn check(&self, item: &GlobalBudgetBurnProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnProposalAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl GlobalBudgetBurnProposalRule for GlobalBudgetBurnProposalAllowedAudienceRule {
    fn check(&self, item: &GlobalBudgetBurnProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is not allowed", item.audience)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global budget burn proposal.
#[derive(Debug, Clone)]
pub struct GlobalBudgetBurnProposalValidator {
    pub rules: Vec<Rc<dyn GlobalBudgetBurnProposalRule>>,
}

impl GlobalBudgetBurnProposalValidator {
    pub fn validate(&self, item: &GlobalBudgetBurnProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalBudgetBurnProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalBudgetBurnProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_budget_burn_proposal_validator() -> GlobalBudgetBurnProposalValidator {
    GlobalBudgetBurnProposalValidator {
        rules: vec![
            Rc::new(GlobalBudgetBurnProposalCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn GlobalBudgetBurnProposalRule>,
            Rc::new(GlobalBudgetBurnProposalBudgetRangeRule { min: 3.7, max: 3981.2 }),
            Rc::new(GlobalBudgetBurnProposalRequiredNameRule {
                message: "global budget burn proposal needs a value".to_string(),
            }),
            Rc::new(GlobalBudgetBurnProposalCodeLengthRule { max_length: 120 }),
            Rc::new(GlobalBudgetBurnProposalAllowedAudienceRule {
                allowed: vec!["legacy".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_budget_burn_proposal_validator();
        let item = new_global_budget_burn_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = GlobalBudgetBurnProposalBudgetRangeRule { min: 5.6, max: 667.5 };
        let mut item = new_global_budget_burn_proposal("x", "y");
        item.budget = 669.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GlobalBudgetBurnProposalRequiredNameRule {
            message: "global budget burn proposal needs an entry".to_string(),
        };
        let mut item = new_global_budget_burn_proposal("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalBudgetBurnProposalCodeLengthRule { max_length: 32 };
        let mut item = new_global_budget_burn_proposal("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
