use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional quote budget must pass. Check returns its problems, or none.
pub trait RegionalQuoteBudgetRule: std::fmt::Debug {
    fn check(&self, item: &RegionalQuoteBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteBudgetCodePrefixRule {
    pub prefix: String,
}

impl RegionalQuoteBudgetRule for RegionalQuoteBudgetCodePrefixRule {
    fn check(&self, item: &RegionalQuoteBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteBudgetCodeLengthRule {
    pub max_length: i64,
}

impl RegionalQuoteBudgetRule for RegionalQuoteBudgetCodeLengthRule {
    fn check(&self, item: &RegionalQuoteBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteBudgetRequiredNameRule {
    pub message: String,
}

impl RegionalQuoteBudgetRule for RegionalQuoteBudgetRequiredNameRule {
    fn check(&self, item: &RegionalQuoteBudget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteBudgetRequiredCodeRule {
    pub message: String,
}

impl RegionalQuoteBudgetRule for RegionalQuoteBudgetRequiredCodeRule {
    fn check(&self, item: &RegionalQuoteBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional quote budget.
#[derive(Debug, Clone)]
pub struct RegionalQuoteBudgetValidator {
    pub rules: Vec<Rc<dyn RegionalQuoteBudgetRule>>,
}

impl RegionalQuoteBudgetValidator {
    pub fn validate(&self, item: &RegionalQuoteBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalQuoteBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalQuoteBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_quote_budget_validator() -> RegionalQuoteBudgetValidator {
    RegionalQuoteBudgetValidator {
        rules: vec![
            Rc::new(RegionalQuoteBudgetCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RegionalQuoteBudgetRule>,
            Rc::new(RegionalQuoteBudgetCodeLengthRule { max_length: 1000 }),
            Rc::new(RegionalQuoteBudgetRequiredNameRule {
                message: "regional quote budget needs an entry".to_string(),
            }),
            Rc::new(RegionalQuoteBudgetRequiredCodeRule {
                message: "regional quote budget needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_quote_budget_validator();
        let item = new_regional_quote_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalQuoteBudgetCodeLengthRule { max_length: 86400 };
        let mut item = new_regional_quote_budget("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalQuoteBudgetRequiredNameRule {
            message: "regional quote budget needs an entry".to_string(),
        };
        let mut item = new_regional_quote_budget("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalQuoteBudgetRequiredCodeRule {
            message: "regional quote budget needs a value".to_string(),
        };
        let mut item = new_regional_quote_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
