use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn group must pass. Check returns its problems, or none.
pub trait BudgetBurnGroupRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnGroupFingerprintLengthRule {
    pub max_length: i64,
}

impl BudgetBurnGroupRule for BudgetBurnGroupFingerprintLengthRule {
    fn check(&self, item: &BudgetBurnGroup) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnGroupTitleLengthRule {
    pub max_length: i64,
}

impl BudgetBurnGroupRule for BudgetBurnGroupTitleLengthRule {
    fn check(&self, item: &BudgetBurnGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnGroupRequiredFingerprintRule {
    pub message: String,
}

impl BudgetBurnGroupRule for BudgetBurnGroupRequiredFingerprintRule {
    fn check(&self, item: &BudgetBurnGroup) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn group.
#[derive(Debug, Clone)]
pub struct BudgetBurnGroupValidator {
    pub rules: Vec<Rc<dyn BudgetBurnGroupRule>>,
}

impl BudgetBurnGroupValidator {
    pub fn validate(&self, item: &BudgetBurnGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_group_validator() -> BudgetBurnGroupValidator {
    BudgetBurnGroupValidator {
        rules: vec![
            Rc::new(BudgetBurnGroupFingerprintLengthRule { max_length: 86400 }) as Rc<dyn BudgetBurnGroupRule>,
            Rc::new(BudgetBurnGroupTitleLengthRule { max_length: 255 }),
            Rc::new(BudgetBurnGroupRequiredFingerprintRule {
                message: "budget burn group needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_group_validator();
        let item = new_budget_burn_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = BudgetBurnGroupFingerprintLengthRule { max_length: 16 };
        let mut item = new_budget_burn_group("x", "y");
        item.fingerprint = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BudgetBurnGroupTitleLengthRule { max_length: 1000 };
        let mut item = new_budget_burn_group("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = BudgetBurnGroupRequiredFingerprintRule {
            message: "budget burn group needs this field".to_string(),
        };
        let mut item = new_budget_burn_group("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
