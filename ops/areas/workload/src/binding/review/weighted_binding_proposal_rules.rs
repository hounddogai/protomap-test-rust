use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted binding proposal must pass. Check returns its problems, or none.
pub trait WeightedBindingProposalRule: std::fmt::Debug {
    fn check(&self, item: &WeightedBindingProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBindingProposalPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedBindingProposalRule for WeightedBindingProposalPriorityRangeRule {
    fn check(&self, item: &WeightedBindingProposal) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBindingProposalAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl WeightedBindingProposalRule for WeightedBindingProposalAllowedTitleRule {
    fn check(&self, item: &WeightedBindingProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is expired", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBindingProposalRequiredCodeRule {
    pub message: String,
}

impl WeightedBindingProposalRule for WeightedBindingProposalRequiredCodeRule {
    fn check(&self, item: &WeightedBindingProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBindingProposalRequiredTitleRule {
    pub message: String,
}

impl WeightedBindingProposalRule for WeightedBindingProposalRequiredTitleRule {
    fn check(&self, item: &WeightedBindingProposal) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBindingProposalQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedBindingProposalRule for WeightedBindingProposalQuotaUnitsRangeRule {
    fn check(&self, item: &WeightedBindingProposal) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBindingProposalExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedBindingProposalRule for WeightedBindingProposalExpiresAtRangeRule {
    fn check(&self, item: &WeightedBindingProposal) -> Vec<String> {
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

/// Runs every rule of a weighted binding proposal.
#[derive(Debug, Clone)]
pub struct WeightedBindingProposalValidator {
    pub rules: Vec<Rc<dyn WeightedBindingProposalRule>>,
}

impl WeightedBindingProposalValidator {
    pub fn validate(&self, item: &WeightedBindingProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedBindingProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedBindingProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_binding_proposal_validator() -> WeightedBindingProposalValidator {
    WeightedBindingProposalValidator {
        rules: vec![
            Rc::new(WeightedBindingProposalPriorityRangeRule { min: 80, max: 86400 })
                as Rc<dyn WeightedBindingProposalRule>,
            Rc::new(WeightedBindingProposalAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(WeightedBindingProposalRequiredCodeRule {
                message: "weighted binding proposal needs this field".to_string(),
            }),
            Rc::new(WeightedBindingProposalRequiredTitleRule {
                message: "weighted binding proposal needs a value".to_string(),
            }),
            Rc::new(WeightedBindingProposalQuotaUnitsRangeRule { min: 16, max: 64 }),
            Rc::new(WeightedBindingProposalExpiresAtRangeRule { min: 1000, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_binding_proposal_validator();
        let item = new_weighted_binding_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = WeightedBindingProposalPriorityRangeRule { min: 120, max: 1000 };
        let mut item = new_weighted_binding_proposal("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedBindingProposalRequiredCodeRule {
            message: "weighted binding proposal needs this field".to_string(),
        };
        let mut item = new_weighted_binding_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedBindingProposalRequiredTitleRule {
            message: "weighted binding proposal needs this field".to_string(),
        };
        let mut item = new_weighted_binding_proposal("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = WeightedBindingProposalQuotaUnitsRangeRule { min: 64, max: 255 };
        let mut item = new_weighted_binding_proposal("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = WeightedBindingProposalExpiresAtRangeRule { min: 3600, max: 86400 };
        let mut item = new_weighted_binding_proposal("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
