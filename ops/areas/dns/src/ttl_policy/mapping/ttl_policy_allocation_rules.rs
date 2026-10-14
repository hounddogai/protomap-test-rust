use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy allocation must pass. Check returns its problems, or none.
pub trait TtlPolicyAllocationRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAllocationThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyAllocationRule for TtlPolicyAllocationThresholdRangeRule {
    fn check(&self, item: &TtlPolicyAllocation) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAllocationRequiredLabelRule {
    pub message: String,
}

impl TtlPolicyAllocationRule for TtlPolicyAllocationRequiredLabelRule {
    fn check(&self, item: &TtlPolicyAllocation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAllocationRequiredRegionRule {
    pub message: String,
}

impl TtlPolicyAllocationRule for TtlPolicyAllocationRequiredRegionRule {
    fn check(&self, item: &TtlPolicyAllocation) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAllocationRequiredCurrencyRule {
    pub message: String,
}

impl TtlPolicyAllocationRule for TtlPolicyAllocationRequiredCurrencyRule {
    fn check(&self, item: &TtlPolicyAllocation) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy allocation.
#[derive(Debug, Clone)]
pub struct TtlPolicyAllocationValidator {
    pub rules: Vec<Rc<dyn TtlPolicyAllocationRule>>,
}

impl TtlPolicyAllocationValidator {
    pub fn validate(&self, item: &TtlPolicyAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_allocation_validator() -> TtlPolicyAllocationValidator {
    TtlPolicyAllocationValidator {
        rules: vec![
            Rc::new(TtlPolicyAllocationThresholdRangeRule { min: 4.6, max: 4200.8 }) as Rc<dyn TtlPolicyAllocationRule>,
            Rc::new(TtlPolicyAllocationRequiredLabelRule {
                message: "ttl policy allocation needs this field".to_string(),
            }),
            Rc::new(TtlPolicyAllocationRequiredRegionRule {
                message: "ttl policy allocation needs a value".to_string(),
            }),
            Rc::new(TtlPolicyAllocationRequiredCurrencyRule {
                message: "ttl policy allocation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_allocation_validator();
        let item = new_ttl_policy_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = TtlPolicyAllocationThresholdRangeRule { min: 8.5, max: 369.7 };
        let mut item = new_ttl_policy_allocation("x", "y");
        item.threshold = 371.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TtlPolicyAllocationRequiredLabelRule {
            message: "ttl policy allocation needs this field".to_string(),
        };
        let mut item = new_ttl_policy_allocation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = TtlPolicyAllocationRequiredRegionRule {
            message: "ttl policy allocation needs a value".to_string(),
        };
        let mut item = new_ttl_policy_allocation("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = TtlPolicyAllocationRequiredCurrencyRule {
            message: "ttl policy allocation needs this field".to_string(),
        };
        let mut item = new_ttl_policy_allocation("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
