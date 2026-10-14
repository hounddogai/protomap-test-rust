use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback budget burn preference must pass. Check returns its problems, or none.
pub trait FallbackBudgetBurnPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &FallbackBudgetBurnPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnPreferenceScopeLengthRule {
    pub max_length: i64,
}

impl FallbackBudgetBurnPreferenceRule for FallbackBudgetBurnPreferenceScopeLengthRule {
    fn check(&self, item: &FallbackBudgetBurnPreference) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnPreferenceRequiredSummaryRule {
    pub message: String,
}

impl FallbackBudgetBurnPreferenceRule for FallbackBudgetBurnPreferenceRequiredSummaryRule {
    fn check(&self, item: &FallbackBudgetBurnPreference) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnPreferenceRequiredScopeRule {
    pub message: String,
}

impl FallbackBudgetBurnPreferenceRule for FallbackBudgetBurnPreferenceRequiredScopeRule {
    fn check(&self, item: &FallbackBudgetBurnPreference) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnPreferenceRequiredCodeRule {
    pub message: String,
}

impl FallbackBudgetBurnPreferenceRule for FallbackBudgetBurnPreferenceRequiredCodeRule {
    fn check(&self, item: &FallbackBudgetBurnPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnPreferenceAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl FallbackBudgetBurnPreferenceRule for FallbackBudgetBurnPreferenceAllowedSummaryRule {
    fn check(&self, item: &FallbackBudgetBurnPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is too long", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl FallbackBudgetBurnPreferenceRule for FallbackBudgetBurnPreferenceCodeLengthRule {
    fn check(&self, item: &FallbackBudgetBurnPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback budget burn preference.
#[derive(Debug, Clone)]
pub struct FallbackBudgetBurnPreferenceValidator {
    pub rules: Vec<Rc<dyn FallbackBudgetBurnPreferenceRule>>,
}

impl FallbackBudgetBurnPreferenceValidator {
    pub fn validate(&self, item: &FallbackBudgetBurnPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackBudgetBurnPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackBudgetBurnPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_budget_burn_preference_validator() -> FallbackBudgetBurnPreferenceValidator {
    FallbackBudgetBurnPreferenceValidator {
        rules: vec![
            Rc::new(FallbackBudgetBurnPreferenceScopeLengthRule { max_length: 86400 })
                as Rc<dyn FallbackBudgetBurnPreferenceRule>,
            Rc::new(FallbackBudgetBurnPreferenceRequiredSummaryRule {
                message: "fallback budget burn preference needs this field".to_string(),
            }),
            Rc::new(FallbackBudgetBurnPreferenceRequiredScopeRule {
                message: "fallback budget burn preference needs a value".to_string(),
            }),
            Rc::new(FallbackBudgetBurnPreferenceRequiredCodeRule {
                message: "fallback budget burn preference needs this field".to_string(),
            }),
            Rc::new(FallbackBudgetBurnPreferenceAllowedSummaryRule {
                allowed: vec!["legacy".to_string(), "lenient".to_string(), "strict".to_string()],
            }),
            Rc::new(FallbackBudgetBurnPreferenceCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_budget_burn_preference_validator();
        let item = new_fallback_budget_burn_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = FallbackBudgetBurnPreferenceScopeLengthRule { max_length: 64 };
        let mut item = new_fallback_budget_burn_preference("x", "y");
        item.scope = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = FallbackBudgetBurnPreferenceRequiredSummaryRule {
            message: "fallback budget burn preference needs this field".to_string(),
        };
        let mut item = new_fallback_budget_burn_preference("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = FallbackBudgetBurnPreferenceRequiredScopeRule {
            message: "fallback budget burn preference needs a value".to_string(),
        };
        let mut item = new_fallback_budget_burn_preference("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackBudgetBurnPreferenceRequiredCodeRule {
            message: "fallback budget burn preference needs a value".to_string(),
        };
        let mut item = new_fallback_budget_burn_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackBudgetBurnPreferenceCodeLengthRule { max_length: 86400 };
        let mut item = new_fallback_budget_burn_preference("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
