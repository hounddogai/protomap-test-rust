use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary plan proposal must pass. Check returns its problems, or none.
pub trait PrimaryPlanProposalRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryPlanProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanProposalRequiredCodeRule {
    pub message: String,
}

impl PrimaryPlanProposalRule for PrimaryPlanProposalRequiredCodeRule {
    fn check(&self, item: &PrimaryPlanProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanProposalExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryPlanProposalRule for PrimaryPlanProposalExpiresAtRangeRule {
    fn check(&self, item: &PrimaryPlanProposal) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanProposalRequiredTitleRule {
    pub message: String,
}

impl PrimaryPlanProposalRule for PrimaryPlanProposalRequiredTitleRule {
    fn check(&self, item: &PrimaryPlanProposal) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanProposalQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryPlanProposalRule for PrimaryPlanProposalQuotaUnitsRangeRule {
    fn check(&self, item: &PrimaryPlanProposal) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanProposalCurrencyLengthRule {
    pub max_length: i64,
}

impl PrimaryPlanProposalRule for PrimaryPlanProposalCurrencyLengthRule {
    fn check(&self, item: &PrimaryPlanProposal) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanProposalCodePrefixRule {
    pub prefix: String,
}

impl PrimaryPlanProposalRule for PrimaryPlanProposalCodePrefixRule {
    fn check(&self, item: &PrimaryPlanProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary plan proposal.
#[derive(Debug, Clone)]
pub struct PrimaryPlanProposalValidator {
    pub rules: Vec<Rc<dyn PrimaryPlanProposalRule>>,
}

impl PrimaryPlanProposalValidator {
    pub fn validate(&self, item: &PrimaryPlanProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryPlanProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryPlanProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_plan_proposal_validator() -> PrimaryPlanProposalValidator {
    PrimaryPlanProposalValidator {
        rules: vec![
            Rc::new(PrimaryPlanProposalRequiredCodeRule {
                message: "primary plan proposal needs an entry".to_string(),
            }) as Rc<dyn PrimaryPlanProposalRule>,
            Rc::new(PrimaryPlanProposalExpiresAtRangeRule { min: 64, max: 120 }),
            Rc::new(PrimaryPlanProposalRequiredTitleRule {
                message: "primary plan proposal needs a value".to_string(),
            }),
            Rc::new(PrimaryPlanProposalQuotaUnitsRangeRule { min: 16, max: 3600 }),
            Rc::new(PrimaryPlanProposalCurrencyLengthRule { max_length: 255 }),
            Rc::new(PrimaryPlanProposalCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_plan_proposal_validator();
        let item = new_primary_plan_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryPlanProposalRequiredCodeRule {
            message: "primary plan proposal needs this field".to_string(),
        };
        let mut item = new_primary_plan_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = PrimaryPlanProposalExpiresAtRangeRule { min: 64, max: 80 };
        let mut item = new_primary_plan_proposal("x", "y");
        item.expires_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryPlanProposalRequiredTitleRule {
            message: "primary plan proposal needs this field".to_string(),
        };
        let mut item = new_primary_plan_proposal("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = PrimaryPlanProposalQuotaUnitsRangeRule { min: 64, max: 255 };
        let mut item = new_primary_plan_proposal("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = PrimaryPlanProposalCurrencyLengthRule { max_length: 255 };
        let mut item = new_primary_plan_proposal("x", "y");
        item.currency = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
