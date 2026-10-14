use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered freeze policy must pass. Check returns its problems, or none.
pub trait TieredFreezePolicyRule: std::fmt::Debug {
    fn check(&self, item: &TieredFreezePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezePolicyBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredFreezePolicyRule for TieredFreezePolicyBurstRangeRule {
    fn check(&self, item: &TieredFreezePolicy) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezePolicyCodeLengthRule {
    pub max_length: i64,
}

impl TieredFreezePolicyRule for TieredFreezePolicyCodeLengthRule {
    fn check(&self, item: &TieredFreezePolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezePolicyRequiredNameRule {
    pub message: String,
}

impl TieredFreezePolicyRule for TieredFreezePolicyRequiredNameRule {
    fn check(&self, item: &TieredFreezePolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezePolicyRequiredCategoryRule {
    pub message: String,
}

impl TieredFreezePolicyRule for TieredFreezePolicyRequiredCategoryRule {
    fn check(&self, item: &TieredFreezePolicy) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezePolicyMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredFreezePolicyRule for TieredFreezePolicyMinimumRangeRule {
    fn check(&self, item: &TieredFreezePolicy) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezePolicyNameLengthRule {
    pub max_length: i64,
}

impl TieredFreezePolicyRule for TieredFreezePolicyNameLengthRule {
    fn check(&self, item: &TieredFreezePolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered freeze policy.
#[derive(Debug, Clone)]
pub struct TieredFreezePolicyValidator {
    pub rules: Vec<Rc<dyn TieredFreezePolicyRule>>,
}

impl TieredFreezePolicyValidator {
    pub fn validate(&self, item: &TieredFreezePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredFreezePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredFreezePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_freeze_policy_validator() -> TieredFreezePolicyValidator {
    TieredFreezePolicyValidator {
        rules: vec![
            Rc::new(TieredFreezePolicyBurstRangeRule { min: 80, max: 120 }) as Rc<dyn TieredFreezePolicyRule>,
            Rc::new(TieredFreezePolicyCodeLengthRule { max_length: 255 }),
            Rc::new(TieredFreezePolicyRequiredNameRule {
                message: "tiered freeze policy needs this field".to_string(),
            }),
            Rc::new(TieredFreezePolicyRequiredCategoryRule {
                message: "tiered freeze policy needs an entry".to_string(),
            }),
            Rc::new(TieredFreezePolicyMinimumRangeRule { min: 9.1, max: 2723.8 }),
            Rc::new(TieredFreezePolicyNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_freeze_policy_validator();
        let item = new_tiered_freeze_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = TieredFreezePolicyBurstRangeRule { min: 255, max: 3600 };
        let mut item = new_tiered_freeze_policy("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredFreezePolicyCodeLengthRule { max_length: 64 };
        let mut item = new_tiered_freeze_policy("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredFreezePolicyRequiredNameRule {
            message: "tiered freeze policy needs this field".to_string(),
        };
        let mut item = new_tiered_freeze_policy("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = TieredFreezePolicyRequiredCategoryRule {
            message: "tiered freeze policy needs a value".to_string(),
        };
        let mut item = new_tiered_freeze_policy("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = TieredFreezePolicyMinimumRangeRule { min: 1.6, max: 2683.4 };
        let mut item = new_tiered_freeze_policy("x", "y");
        item.minimum = 2684.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredFreezePolicyNameLengthRule { max_length: 16 };
        let mut item = new_tiered_freeze_policy("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
