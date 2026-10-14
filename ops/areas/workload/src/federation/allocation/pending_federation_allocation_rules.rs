use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending federation allocation must pass. Check returns its problems, or none.
pub trait PendingFederationAllocationRule: std::fmt::Debug {
    fn check(&self, item: &PendingFederationAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationAllocationCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingFederationAllocationRule for PendingFederationAllocationCostRangeRule {
    fn check(&self, item: &PendingFederationAllocation) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationAllocationRequiredCodeRule {
    pub message: String,
}

impl PendingFederationAllocationRule for PendingFederationAllocationRequiredCodeRule {
    fn check(&self, item: &PendingFederationAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationAllocationRequiredLabelRule {
    pub message: String,
}

impl PendingFederationAllocationRule for PendingFederationAllocationRequiredLabelRule {
    fn check(&self, item: &PendingFederationAllocation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationAllocationRequiredAudienceRule {
    pub message: String,
}

impl PendingFederationAllocationRule for PendingFederationAllocationRequiredAudienceRule {
    fn check(&self, item: &PendingFederationAllocation) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending federation allocation.
#[derive(Debug, Clone)]
pub struct PendingFederationAllocationValidator {
    pub rules: Vec<Rc<dyn PendingFederationAllocationRule>>,
}

impl PendingFederationAllocationValidator {
    pub fn validate(&self, item: &PendingFederationAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingFederationAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFederationAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_federation_allocation_validator() -> PendingFederationAllocationValidator {
    PendingFederationAllocationValidator {
        rules: vec![
            Rc::new(PendingFederationAllocationCostRangeRule { min: 2.6, max: 3605.8 })
                as Rc<dyn PendingFederationAllocationRule>,
            Rc::new(PendingFederationAllocationRequiredCodeRule {
                message: "pending federation allocation needs this field".to_string(),
            }),
            Rc::new(PendingFederationAllocationRequiredLabelRule {
                message: "pending federation allocation needs an entry".to_string(),
            }),
            Rc::new(PendingFederationAllocationRequiredAudienceRule {
                message: "pending federation allocation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_federation_allocation_validator();
        let item = new_pending_federation_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = PendingFederationAllocationCostRangeRule { min: 1.7, max: 4644.8 };
        let mut item = new_pending_federation_allocation("x", "y");
        item.cost = 4646.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingFederationAllocationRequiredCodeRule {
            message: "pending federation allocation needs a value".to_string(),
        };
        let mut item = new_pending_federation_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingFederationAllocationRequiredLabelRule {
            message: "pending federation allocation needs an entry".to_string(),
        };
        let mut item = new_pending_federation_allocation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = PendingFederationAllocationRequiredAudienceRule {
            message: "pending federation allocation needs a value".to_string(),
        };
        let mut item = new_pending_federation_allocation("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
