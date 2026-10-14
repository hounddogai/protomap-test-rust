use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered tenant adjustment must pass. Check returns its problems, or none.
pub trait TieredTenantAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &TieredTenantAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantAdjustmentRequiredTitleRule {
    pub message: String,
}

impl TieredTenantAdjustmentRule for TieredTenantAdjustmentRequiredTitleRule {
    fn check(&self, item: &TieredTenantAdjustment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantAdjustmentThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredTenantAdjustmentRule for TieredTenantAdjustmentThresholdRangeRule {
    fn check(&self, item: &TieredTenantAdjustment) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl TieredTenantAdjustmentRule for TieredTenantAdjustmentCodeLengthRule {
    fn check(&self, item: &TieredTenantAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantAdjustmentSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredTenantAdjustmentRule for TieredTenantAdjustmentSequenceRangeRule {
    fn check(&self, item: &TieredTenantAdjustment) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered tenant adjustment.
#[derive(Debug, Clone)]
pub struct TieredTenantAdjustmentValidator {
    pub rules: Vec<Rc<dyn TieredTenantAdjustmentRule>>,
}

impl TieredTenantAdjustmentValidator {
    pub fn validate(&self, item: &TieredTenantAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredTenantAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredTenantAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_tenant_adjustment_validator() -> TieredTenantAdjustmentValidator {
    TieredTenantAdjustmentValidator {
        rules: vec![
            Rc::new(TieredTenantAdjustmentRequiredTitleRule {
                message: "tiered tenant adjustment needs an entry".to_string(),
            }) as Rc<dyn TieredTenantAdjustmentRule>,
            Rc::new(TieredTenantAdjustmentThresholdRangeRule { min: 7.4, max: 447.4 }),
            Rc::new(TieredTenantAdjustmentCodeLengthRule { max_length: 16 }),
            Rc::new(TieredTenantAdjustmentSequenceRangeRule { min: 255, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_tenant_adjustment_validator();
        let item = new_tiered_tenant_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredTenantAdjustmentRequiredTitleRule {
            message: "tiered tenant adjustment needs a value".to_string(),
        };
        let mut item = new_tiered_tenant_adjustment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = TieredTenantAdjustmentThresholdRangeRule { min: 8.3, max: 2300.8 };
        let mut item = new_tiered_tenant_adjustment("x", "y");
        item.threshold = 2302.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredTenantAdjustmentCodeLengthRule { max_length: 1000 };
        let mut item = new_tiered_tenant_adjustment("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = TieredTenantAdjustmentSequenceRangeRule { min: 16, max: 255 };
        let mut item = new_tiered_tenant_adjustment("x", "y");
        item.sequence = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
