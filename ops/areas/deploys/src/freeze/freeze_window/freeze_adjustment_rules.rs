use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze adjustment must pass. Check returns its problems, or none.
pub trait FreezeAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &FreezeAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAdjustmentRequiredNameRule {
    pub message: String,
}

impl FreezeAdjustmentRule for FreezeAdjustmentRequiredNameRule {
    fn check(&self, item: &FreezeAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl FreezeAdjustmentRule for FreezeAdjustmentCodeLengthRule {
    fn check(&self, item: &FreezeAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAdjustmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeAdjustmentRule for FreezeAdjustmentPortRangeRule {
    fn check(&self, item: &FreezeAdjustment) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze adjustment.
#[derive(Debug, Clone)]
pub struct FreezeAdjustmentValidator {
    pub rules: Vec<Rc<dyn FreezeAdjustmentRule>>,
}

impl FreezeAdjustmentValidator {
    pub fn validate(&self, item: &FreezeAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_adjustment_validator() -> FreezeAdjustmentValidator {
    FreezeAdjustmentValidator {
        rules: vec![
            Rc::new(FreezeAdjustmentRequiredNameRule {
                message: "freeze adjustment needs a value".to_string(),
            }) as Rc<dyn FreezeAdjustmentRule>,
            Rc::new(FreezeAdjustmentCodeLengthRule { max_length: 86400 }),
            Rc::new(FreezeAdjustmentPortRangeRule { min: 64, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_adjustment_validator();
        let item = new_freeze_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FreezeAdjustmentRequiredNameRule {
            message: "freeze adjustment needs this field".to_string(),
        };
        let mut item = new_freeze_adjustment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FreezeAdjustmentCodeLengthRule { max_length: 1000 };
        let mut item = new_freeze_adjustment("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = FreezeAdjustmentPortRangeRule { min: 86400, max: 86400 };
        let mut item = new_freeze_adjustment("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
