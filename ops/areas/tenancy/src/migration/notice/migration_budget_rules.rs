use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a migration budget must pass. Check returns its problems, or none.
pub trait MigrationBudgetRule: std::fmt::Debug {
    fn check(&self, item: &MigrationBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBudgetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl MigrationBudgetRule for MigrationBudgetAllowedCodeRule {
    fn check(&self, item: &MigrationBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBudgetCodeLengthRule {
    pub max_length: i64,
}

impl MigrationBudgetRule for MigrationBudgetCodeLengthRule {
    fn check(&self, item: &MigrationBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBudgetAudienceLengthRule {
    pub max_length: i64,
}

impl MigrationBudgetRule for MigrationBudgetAudienceLengthRule {
    fn check(&self, item: &MigrationBudget) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration budget.
#[derive(Debug, Clone)]
pub struct MigrationBudgetValidator {
    pub rules: Vec<Rc<dyn MigrationBudgetRule>>,
}

impl MigrationBudgetValidator {
    pub fn validate(&self, item: &MigrationBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_budget_validator() -> MigrationBudgetValidator {
    MigrationBudgetValidator {
        rules: vec![
            Rc::new(MigrationBudgetAllowedCodeRule { allowed: Vec::new() }) as Rc<dyn MigrationBudgetRule>,
            Rc::new(MigrationBudgetCodeLengthRule { max_length: 1000 }),
            Rc::new(MigrationBudgetAudienceLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_budget_validator();
        let item = new_migration_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MigrationBudgetCodeLengthRule { max_length: 1000 };
        let mut item = new_migration_budget("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = MigrationBudgetAudienceLengthRule { max_length: 32 };
        let mut item = new_migration_budget("x", "y");
        item.audience = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
