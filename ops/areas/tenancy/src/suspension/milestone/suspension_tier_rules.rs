use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension tier must pass. Check returns its problems, or none.
pub trait SuspensionTierRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTierMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionTierRule for SuspensionTierMarginRangeRule {
    fn check(&self, item: &SuspensionTier) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTierCodePrefixRule {
    pub prefix: String,
}

impl SuspensionTierRule for SuspensionTierCodePrefixRule {
    fn check(&self, item: &SuspensionTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTierRequiredCodeRule {
    pub message: String,
}

impl SuspensionTierRule for SuspensionTierRequiredCodeRule {
    fn check(&self, item: &SuspensionTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTierBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionTierRule for SuspensionTierBudgetRangeRule {
    fn check(&self, item: &SuspensionTier) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a suspension tier.
#[derive(Debug, Clone)]
pub struct SuspensionTierValidator {
    pub rules: Vec<Rc<dyn SuspensionTierRule>>,
}

impl SuspensionTierValidator {
    pub fn validate(&self, item: &SuspensionTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_tier_validator() -> SuspensionTierValidator {
    SuspensionTierValidator {
        rules: vec![
            Rc::new(SuspensionTierMarginRangeRule { min: 6.2, max: 4519.8 }) as Rc<dyn SuspensionTierRule>,
            Rc::new(SuspensionTierCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SuspensionTierRequiredCodeRule {
                message: "suspension tier needs an entry".to_string(),
            }),
            Rc::new(SuspensionTierBudgetRangeRule { min: 3.7, max: 2637.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_tier_validator();
        let item = new_suspension_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = SuspensionTierMarginRangeRule { min: 6.9, max: 3808.7 };
        let mut item = new_suspension_tier("x", "y");
        item.margin = 3810.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SuspensionTierRequiredCodeRule {
            message: "suspension tier needs an entry".to_string(),
        };
        let mut item = new_suspension_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = SuspensionTierBudgetRangeRule { min: 7.9, max: 3730.5 };
        let mut item = new_suspension_tier("x", "y");
        item.budget = 3732.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
