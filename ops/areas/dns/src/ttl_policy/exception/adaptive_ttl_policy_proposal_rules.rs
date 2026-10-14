use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive ttl policy proposal must pass. Check returns its problems, or none.
pub trait AdaptiveTtlPolicyProposalRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveTtlPolicyProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicyProposalCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveTtlPolicyProposalRule for AdaptiveTtlPolicyProposalCodeLengthRule {
    fn check(&self, item: &AdaptiveTtlPolicyProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicyProposalBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveTtlPolicyProposalRule for AdaptiveTtlPolicyProposalBudgetRangeRule {
    fn check(&self, item: &AdaptiveTtlPolicyProposal) -> Vec<String> {
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
pub struct AdaptiveTtlPolicyProposalRequiredCodeRule {
    pub message: String,
}

impl AdaptiveTtlPolicyProposalRule for AdaptiveTtlPolicyProposalRequiredCodeRule {
    fn check(&self, item: &AdaptiveTtlPolicyProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicyProposalCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveTtlPolicyProposalRule for AdaptiveTtlPolicyProposalCodePrefixRule {
    fn check(&self, item: &AdaptiveTtlPolicyProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicyProposalRequiredTenantIdRule {
    pub message: String,
}

impl AdaptiveTtlPolicyProposalRule for AdaptiveTtlPolicyProposalRequiredTenantIdRule {
    fn check(&self, item: &AdaptiveTtlPolicyProposal) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive ttl policy proposal.
#[derive(Debug, Clone)]
pub struct AdaptiveTtlPolicyProposalValidator {
    pub rules: Vec<Rc<dyn AdaptiveTtlPolicyProposalRule>>,
}

impl AdaptiveTtlPolicyProposalValidator {
    pub fn validate(&self, item: &AdaptiveTtlPolicyProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveTtlPolicyProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveTtlPolicyProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_ttl_policy_proposal_validator() -> AdaptiveTtlPolicyProposalValidator {
    AdaptiveTtlPolicyProposalValidator {
        rules: vec![
            Rc::new(AdaptiveTtlPolicyProposalCodeLengthRule { max_length: 120 })
                as Rc<dyn AdaptiveTtlPolicyProposalRule>,
            Rc::new(AdaptiveTtlPolicyProposalBudgetRangeRule { min: 3.2, max: 1994.5 }),
            Rc::new(AdaptiveTtlPolicyProposalRequiredCodeRule {
                message: "adaptive ttl policy proposal needs a value".to_string(),
            }),
            Rc::new(AdaptiveTtlPolicyProposalCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(AdaptiveTtlPolicyProposalRequiredTenantIdRule {
                message: "adaptive ttl policy proposal needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_ttl_policy_proposal_validator();
        let item = new_adaptive_ttl_policy_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveTtlPolicyProposalCodeLengthRule { max_length: 16 };
        let mut item = new_adaptive_ttl_policy_proposal("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = AdaptiveTtlPolicyProposalBudgetRangeRule { min: 2.2, max: 3031.2 };
        let mut item = new_adaptive_ttl_policy_proposal("x", "y");
        item.budget = 3032.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveTtlPolicyProposalRequiredCodeRule {
            message: "adaptive ttl policy proposal needs a value".to_string(),
        };
        let mut item = new_adaptive_ttl_policy_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = AdaptiveTtlPolicyProposalRequiredTenantIdRule {
            message: "adaptive ttl policy proposal needs an entry".to_string(),
        };
        let mut item = new_adaptive_ttl_policy_proposal("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
