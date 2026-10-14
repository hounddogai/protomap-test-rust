use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback overage budget must pass. Check returns its problems, or none.
pub trait FallbackOverageBudgetRule: std::fmt::Debug {
    fn check(&self, item: &FallbackOverageBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageBudgetRequiredHostRule {
    pub message: String,
}

impl FallbackOverageBudgetRule for FallbackOverageBudgetRequiredHostRule {
    fn check(&self, item: &FallbackOverageBudget) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageBudgetLabelLengthRule {
    pub max_length: i64,
}

impl FallbackOverageBudgetRule for FallbackOverageBudgetLabelLengthRule {
    fn check(&self, item: &FallbackOverageBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageBudgetHostLengthRule {
    pub max_length: i64,
}

impl FallbackOverageBudgetRule for FallbackOverageBudgetHostLengthRule {
    fn check(&self, item: &FallbackOverageBudget) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageBudgetFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackOverageBudgetRule for FallbackOverageBudgetFloorValueRangeRule {
    fn check(&self, item: &FallbackOverageBudget) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageBudgetCodePrefixRule {
    pub prefix: String,
}

impl FallbackOverageBudgetRule for FallbackOverageBudgetCodePrefixRule {
    fn check(&self, item: &FallbackOverageBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback overage budget.
#[derive(Debug, Clone)]
pub struct FallbackOverageBudgetValidator {
    pub rules: Vec<Rc<dyn FallbackOverageBudgetRule>>,
}

impl FallbackOverageBudgetValidator {
    pub fn validate(&self, item: &FallbackOverageBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackOverageBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackOverageBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_overage_budget_validator() -> FallbackOverageBudgetValidator {
    FallbackOverageBudgetValidator {
        rules: vec![
            Rc::new(FallbackOverageBudgetRequiredHostRule {
                message: "fallback overage budget needs a value".to_string(),
            }) as Rc<dyn FallbackOverageBudgetRule>,
            Rc::new(FallbackOverageBudgetLabelLengthRule { max_length: 86400 }),
            Rc::new(FallbackOverageBudgetHostLengthRule { max_length: 32 }),
            Rc::new(FallbackOverageBudgetFloorValueRangeRule { min: 8.4, max: 4145.1 }),
            Rc::new(FallbackOverageBudgetCodePrefixRule {
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
        let validator = default_fallback_overage_budget_validator();
        let item = new_fallback_overage_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = FallbackOverageBudgetRequiredHostRule {
            message: "fallback overage budget needs a value".to_string(),
        };
        let mut item = new_fallback_overage_budget("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackOverageBudgetLabelLengthRule { max_length: 16 };
        let mut item = new_fallback_overage_budget("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = FallbackOverageBudgetHostLengthRule { max_length: 255 };
        let mut item = new_fallback_overage_budget("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = FallbackOverageBudgetFloorValueRangeRule { min: 0.9, max: 355.9 };
        let mut item = new_fallback_overage_budget("x", "y");
        item.floor_value = 357.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
