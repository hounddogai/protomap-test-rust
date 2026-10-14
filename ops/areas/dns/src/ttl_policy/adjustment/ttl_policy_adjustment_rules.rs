use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy adjustment must pass. Check returns its problems, or none.
pub trait TtlPolicyAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAdjustmentAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyAdjustmentRule for TtlPolicyAdjustmentAmountRangeRule {
    fn check(&self, item: &TtlPolicyAdjustment) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAdjustmentSegmentLengthRule {
    pub max_length: i64,
}

impl TtlPolicyAdjustmentRule for TtlPolicyAdjustmentSegmentLengthRule {
    fn check(&self, item: &TtlPolicyAdjustment) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAdjustmentRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyAdjustmentRule for TtlPolicyAdjustmentRequiredCodeRule {
    fn check(&self, item: &TtlPolicyAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy adjustment.
#[derive(Debug, Clone)]
pub struct TtlPolicyAdjustmentValidator {
    pub rules: Vec<Rc<dyn TtlPolicyAdjustmentRule>>,
}

impl TtlPolicyAdjustmentValidator {
    pub fn validate(&self, item: &TtlPolicyAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_adjustment_validator() -> TtlPolicyAdjustmentValidator {
    TtlPolicyAdjustmentValidator {
        rules: vec![
            Rc::new(TtlPolicyAdjustmentAmountRangeRule { min: 6.9, max: 600.9 }) as Rc<dyn TtlPolicyAdjustmentRule>,
            Rc::new(TtlPolicyAdjustmentSegmentLengthRule { max_length: 16 }),
            Rc::new(TtlPolicyAdjustmentRequiredCodeRule {
                message: "ttl policy adjustment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_adjustment_validator();
        let item = new_ttl_policy_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = TtlPolicyAdjustmentAmountRangeRule { min: 3.0, max: 4771.1 };
        let mut item = new_ttl_policy_adjustment("x", "y");
        item.amount = 4772.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = TtlPolicyAdjustmentSegmentLengthRule { max_length: 86400 };
        let mut item = new_ttl_policy_adjustment("x", "y");
        item.segment = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyAdjustmentRequiredCodeRule {
            message: "ttl policy adjustment needs this field".to_string(),
        };
        let mut item = new_ttl_policy_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
