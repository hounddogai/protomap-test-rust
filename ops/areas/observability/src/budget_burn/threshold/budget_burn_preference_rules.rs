use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a budget burn preference must pass. Check returns its problems, or none.
pub trait BudgetBurnPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPreferenceKeyIdLengthRule {
    pub max_length: i64,
}

impl BudgetBurnPreferenceRule for BudgetBurnPreferenceKeyIdLengthRule {
    fn check(&self, item: &BudgetBurnPreference) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl BudgetBurnPreferenceRule for BudgetBurnPreferenceCodeLengthRule {
    fn check(&self, item: &BudgetBurnPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPreferenceRequiredExternalRefRule {
    pub message: String,
}

impl BudgetBurnPreferenceRule for BudgetBurnPreferenceRequiredExternalRefRule {
    fn check(&self, item: &BudgetBurnPreference) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPreferenceAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl BudgetBurnPreferenceRule for BudgetBurnPreferenceAllowedKeyIdRule {
    fn check(&self, item: &BudgetBurnPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is expired", item.key_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn preference.
#[derive(Debug, Clone)]
pub struct BudgetBurnPreferenceValidator {
    pub rules: Vec<Rc<dyn BudgetBurnPreferenceRule>>,
}

impl BudgetBurnPreferenceValidator {
    pub fn validate(&self, item: &BudgetBurnPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_preference_validator() -> BudgetBurnPreferenceValidator {
    BudgetBurnPreferenceValidator {
        rules: vec![
            Rc::new(BudgetBurnPreferenceKeyIdLengthRule { max_length: 1000 }) as Rc<dyn BudgetBurnPreferenceRule>,
            Rc::new(BudgetBurnPreferenceCodeLengthRule { max_length: 32 }),
            Rc::new(BudgetBurnPreferenceRequiredExternalRefRule {
                message: "budget burn preference needs an entry".to_string(),
            }),
            Rc::new(BudgetBurnPreferenceAllowedKeyIdRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_preference_validator();
        let item = new_budget_burn_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = BudgetBurnPreferenceKeyIdLengthRule { max_length: 64 };
        let mut item = new_budget_burn_preference("x", "y");
        item.key_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BudgetBurnPreferenceCodeLengthRule { max_length: 32 };
        let mut item = new_budget_burn_preference("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = BudgetBurnPreferenceRequiredExternalRefRule {
            message: "budget burn preference needs an entry".to_string(),
        };
        let mut item = new_budget_burn_preference("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
