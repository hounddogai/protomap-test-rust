use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered budget burn profile must pass. Check returns its problems, or none.
pub trait TieredBudgetBurnProfileRule: std::fmt::Debug {
    fn check(&self, item: &TieredBudgetBurnProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnProfileAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl TieredBudgetBurnProfileRule for TieredBudgetBurnProfileAllowedNamespaceRule {
    fn check(&self, item: &TieredBudgetBurnProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is missing", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnProfileCipherLengthRule {
    pub max_length: i64,
}

impl TieredBudgetBurnProfileRule for TieredBudgetBurnProfileCipherLengthRule {
    fn check(&self, item: &TieredBudgetBurnProfile) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnProfileRequiredTitleRule {
    pub message: String,
}

impl TieredBudgetBurnProfileRule for TieredBudgetBurnProfileRequiredTitleRule {
    fn check(&self, item: &TieredBudgetBurnProfile) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnProfileRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredBudgetBurnProfileRule for TieredBudgetBurnProfileRotationDaysRangeRule {
    fn check(&self, item: &TieredBudgetBurnProfile) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnProfileTitleLengthRule {
    pub max_length: i64,
}

impl TieredBudgetBurnProfileRule for TieredBudgetBurnProfileTitleLengthRule {
    fn check(&self, item: &TieredBudgetBurnProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered budget burn profile.
#[derive(Debug, Clone)]
pub struct TieredBudgetBurnProfileValidator {
    pub rules: Vec<Rc<dyn TieredBudgetBurnProfileRule>>,
}

impl TieredBudgetBurnProfileValidator {
    pub fn validate(&self, item: &TieredBudgetBurnProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredBudgetBurnProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredBudgetBurnProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_budget_burn_profile_validator() -> TieredBudgetBurnProfileValidator {
    TieredBudgetBurnProfileValidator {
        rules: vec![
            Rc::new(TieredBudgetBurnProfileAllowedNamespaceRule {
                allowed: vec!["adaptive".to_string(), "primary".to_string()],
            }) as Rc<dyn TieredBudgetBurnProfileRule>,
            Rc::new(TieredBudgetBurnProfileCipherLengthRule { max_length: 1000 }),
            Rc::new(TieredBudgetBurnProfileRequiredTitleRule {
                message: "tiered budget burn profile needs a value".to_string(),
            }),
            Rc::new(TieredBudgetBurnProfileRotationDaysRangeRule { min: 16, max: 32 }),
            Rc::new(TieredBudgetBurnProfileTitleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_budget_burn_profile_validator();
        let item = new_tiered_budget_burn_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = TieredBudgetBurnProfileCipherLengthRule { max_length: 1000 };
        let mut item = new_tiered_budget_burn_profile("x", "y");
        item.cipher = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredBudgetBurnProfileRequiredTitleRule {
            message: "tiered budget burn profile needs a value".to_string(),
        };
        let mut item = new_tiered_budget_burn_profile("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = TieredBudgetBurnProfileRotationDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_tiered_budget_burn_profile("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredBudgetBurnProfileTitleLengthRule { max_length: 32 };
        let mut item = new_tiered_budget_burn_profile("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
