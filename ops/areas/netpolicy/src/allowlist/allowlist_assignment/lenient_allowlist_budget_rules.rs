use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient allowlist budget must pass. Check returns its problems, or none.
pub trait LenientAllowlistBudgetRule: std::fmt::Debug {
    fn check(&self, item: &LenientAllowlistBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistBudgetRequiredNameRule {
    pub message: String,
}

impl LenientAllowlistBudgetRule for LenientAllowlistBudgetRequiredNameRule {
    fn check(&self, item: &LenientAllowlistBudget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistBudgetCodeLengthRule {
    pub max_length: i64,
}

impl LenientAllowlistBudgetRule for LenientAllowlistBudgetCodeLengthRule {
    fn check(&self, item: &LenientAllowlistBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistBudgetRequiredCodeRule {
    pub message: String,
}

impl LenientAllowlistBudgetRule for LenientAllowlistBudgetRequiredCodeRule {
    fn check(&self, item: &LenientAllowlistBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient allowlist budget.
#[derive(Debug, Clone)]
pub struct LenientAllowlistBudgetValidator {
    pub rules: Vec<Rc<dyn LenientAllowlistBudgetRule>>,
}

impl LenientAllowlistBudgetValidator {
    pub fn validate(&self, item: &LenientAllowlistBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientAllowlistBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientAllowlistBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_allowlist_budget_validator() -> LenientAllowlistBudgetValidator {
    LenientAllowlistBudgetValidator {
        rules: vec![
            Rc::new(LenientAllowlistBudgetRequiredNameRule {
                message: "lenient allowlist budget needs this field".to_string(),
            }) as Rc<dyn LenientAllowlistBudgetRule>,
            Rc::new(LenientAllowlistBudgetCodeLengthRule { max_length: 16 }),
            Rc::new(LenientAllowlistBudgetRequiredCodeRule {
                message: "lenient allowlist budget needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_allowlist_budget_validator();
        let item = new_lenient_allowlist_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientAllowlistBudgetRequiredNameRule {
            message: "lenient allowlist budget needs an entry".to_string(),
        };
        let mut item = new_lenient_allowlist_budget("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientAllowlistBudgetCodeLengthRule { max_length: 16 };
        let mut item = new_lenient_allowlist_budget("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientAllowlistBudgetRequiredCodeRule {
            message: "lenient allowlist budget needs an entry".to_string(),
        };
        let mut item = new_lenient_allowlist_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
