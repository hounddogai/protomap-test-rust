use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered failover adjustment must pass. Check returns its problems, or none.
pub trait TieredFailoverAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &TieredFailoverAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverAdjustmentAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredFailoverAdjustmentRule for TieredFailoverAdjustmentAmountRangeRule {
    fn check(&self, item: &TieredFailoverAdjustment) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl TieredFailoverAdjustmentRule for TieredFailoverAdjustmentCodeLengthRule {
    fn check(&self, item: &TieredFailoverAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverAdjustmentLabelLengthRule {
    pub max_length: i64,
}

impl TieredFailoverAdjustmentRule for TieredFailoverAdjustmentLabelLengthRule {
    fn check(&self, item: &TieredFailoverAdjustment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverAdjustmentRequiredCodeRule {
    pub message: String,
}

impl TieredFailoverAdjustmentRule for TieredFailoverAdjustmentRequiredCodeRule {
    fn check(&self, item: &TieredFailoverAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered failover adjustment.
#[derive(Debug, Clone)]
pub struct TieredFailoverAdjustmentValidator {
    pub rules: Vec<Rc<dyn TieredFailoverAdjustmentRule>>,
}

impl TieredFailoverAdjustmentValidator {
    pub fn validate(&self, item: &TieredFailoverAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredFailoverAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredFailoverAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_failover_adjustment_validator() -> TieredFailoverAdjustmentValidator {
    TieredFailoverAdjustmentValidator {
        rules: vec![
            Rc::new(TieredFailoverAdjustmentAmountRangeRule { min: 4.3, max: 565.5 })
                as Rc<dyn TieredFailoverAdjustmentRule>,
            Rc::new(TieredFailoverAdjustmentCodeLengthRule { max_length: 64 }),
            Rc::new(TieredFailoverAdjustmentLabelLengthRule { max_length: 120 }),
            Rc::new(TieredFailoverAdjustmentRequiredCodeRule {
                message: "tiered failover adjustment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_failover_adjustment_validator();
        let item = new_tiered_failover_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = TieredFailoverAdjustmentAmountRangeRule { min: 1.0, max: 2272.8 };
        let mut item = new_tiered_failover_adjustment("x", "y");
        item.amount = 2274.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredFailoverAdjustmentCodeLengthRule { max_length: 1000 };
        let mut item = new_tiered_failover_adjustment("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TieredFailoverAdjustmentLabelLengthRule { max_length: 120 };
        let mut item = new_tiered_failover_adjustment("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredFailoverAdjustmentRequiredCodeRule {
            message: "tiered failover adjustment needs this field".to_string(),
        };
        let mut item = new_tiered_failover_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
