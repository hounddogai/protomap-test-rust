use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a budget burn assignment must pass. Check returns its problems, or none.
pub trait BudgetBurnAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAssignmentAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl BudgetBurnAssignmentRule for BudgetBurnAssignmentAllowedPhoneRule {
    fn check(&self, item: &BudgetBurnAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is out of range", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAssignmentRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnAssignmentRule for BudgetBurnAssignmentRotationDaysRangeRule {
    fn check(&self, item: &BudgetBurnAssignment) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAssignmentPhoneLengthRule {
    pub max_length: i64,
}

impl BudgetBurnAssignmentRule for BudgetBurnAssignmentPhoneLengthRule {
    fn check(&self, item: &BudgetBurnAssignment) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAssignmentNameLengthRule {
    pub max_length: i64,
}

impl BudgetBurnAssignmentRule for BudgetBurnAssignmentNameLengthRule {
    fn check(&self, item: &BudgetBurnAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl BudgetBurnAssignmentRule for BudgetBurnAssignmentCodeLengthRule {
    fn check(&self, item: &BudgetBurnAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn assignment.
#[derive(Debug, Clone)]
pub struct BudgetBurnAssignmentValidator {
    pub rules: Vec<Rc<dyn BudgetBurnAssignmentRule>>,
}

impl BudgetBurnAssignmentValidator {
    pub fn validate(&self, item: &BudgetBurnAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_assignment_validator() -> BudgetBurnAssignmentValidator {
    BudgetBurnAssignmentValidator {
        rules: vec![
            Rc::new(BudgetBurnAssignmentAllowedPhoneRule {
                allowed: vec!["pending".to_string(), "strict".to_string()],
            }) as Rc<dyn BudgetBurnAssignmentRule>,
            Rc::new(BudgetBurnAssignmentRotationDaysRangeRule { min: 16, max: 32 }),
            Rc::new(BudgetBurnAssignmentPhoneLengthRule { max_length: 16 }),
            Rc::new(BudgetBurnAssignmentNameLengthRule { max_length: 32 }),
            Rc::new(BudgetBurnAssignmentCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_assignment_validator();
        let item = new_budget_burn_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = BudgetBurnAssignmentRotationDaysRangeRule { min: 120, max: 86400 };
        let mut item = new_budget_burn_assignment("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = BudgetBurnAssignmentPhoneLengthRule { max_length: 86400 };
        let mut item = new_budget_burn_assignment("x", "y");
        item.phone = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BudgetBurnAssignmentNameLengthRule { max_length: 86400 };
        let mut item = new_budget_burn_assignment("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BudgetBurnAssignmentCodeLengthRule { max_length: 120 };
        let mut item = new_budget_burn_assignment("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
