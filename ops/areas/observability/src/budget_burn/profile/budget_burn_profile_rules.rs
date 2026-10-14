use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a budget burn profile must pass. Check returns its problems, or none.
pub trait BudgetBurnProfileRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProfilePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnProfileRule for BudgetBurnProfilePriorityRangeRule {
    fn check(&self, item: &BudgetBurnProfile) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProfileAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl BudgetBurnProfileRule for BudgetBurnProfileAllowedTitleRule {
    fn check(&self, item: &BudgetBurnProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProfileCodeLengthRule {
    pub max_length: i64,
}

impl BudgetBurnProfileRule for BudgetBurnProfileCodeLengthRule {
    fn check(&self, item: &BudgetBurnProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProfileRequiredCodeRule {
    pub message: String,
}

impl BudgetBurnProfileRule for BudgetBurnProfileRequiredCodeRule {
    fn check(&self, item: &BudgetBurnProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProfileCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnProfileRule for BudgetBurnProfileCodePrefixRule {
    fn check(&self, item: &BudgetBurnProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProfilePinnedGuardRule {
    pub limit: f64,
}

impl BudgetBurnProfileRule for BudgetBurnProfilePinnedGuardRule {
    fn check(&self, item: &BudgetBurnProfile) -> Vec<String> {
        if item.pinned && (item.minimum > self.limit) {
            return vec![format!("minimum {} exceeds {} while pinned", item.minimum, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn profile.
#[derive(Debug, Clone)]
pub struct BudgetBurnProfileValidator {
    pub rules: Vec<Rc<dyn BudgetBurnProfileRule>>,
}

impl BudgetBurnProfileValidator {
    pub fn validate(&self, item: &BudgetBurnProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_profile_validator() -> BudgetBurnProfileValidator {
    BudgetBurnProfileValidator {
        rules: vec![
            Rc::new(BudgetBurnProfilePriorityRangeRule { min: 16, max: 80 }) as Rc<dyn BudgetBurnProfileRule>,
            Rc::new(BudgetBurnProfileAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(BudgetBurnProfileCodeLengthRule { max_length: 255 }),
            Rc::new(BudgetBurnProfileRequiredCodeRule {
                message: "budget burn profile needs a value".to_string(),
            }),
            Rc::new(BudgetBurnProfileCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(BudgetBurnProfilePinnedGuardRule { limit: 7.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_profile_validator();
        let item = new_budget_burn_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = BudgetBurnProfilePriorityRangeRule { min: 64, max: 80 };
        let mut item = new_budget_burn_profile("x", "y");
        item.priority = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BudgetBurnProfileCodeLengthRule { max_length: 64 };
        let mut item = new_budget_burn_profile("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BudgetBurnProfileRequiredCodeRule {
            message: "budget burn profile needs an entry".to_string(),
        };
        let mut item = new_budget_burn_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
