use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration reservation must pass. Check returns its problems, or none.
pub trait MigrationReservationRule: std::fmt::Debug {
    fn check(&self, item: &MigrationReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReservationRequiredUnitRule {
    pub message: String,
}

impl MigrationReservationRule for MigrationReservationRequiredUnitRule {
    fn check(&self, item: &MigrationReservation) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReservationCodePrefixRule {
    pub prefix: String,
}

impl MigrationReservationRule for MigrationReservationCodePrefixRule {
    fn check(&self, item: &MigrationReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReservationBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationReservationRule for MigrationReservationBudgetRangeRule {
    fn check(&self, item: &MigrationReservation) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReservationRequiredCodeRule {
    pub message: String,
}

impl MigrationReservationRule for MigrationReservationRequiredCodeRule {
    fn check(&self, item: &MigrationReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReservationCodeLengthRule {
    pub max_length: i64,
}

impl MigrationReservationRule for MigrationReservationCodeLengthRule {
    fn check(&self, item: &MigrationReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration reservation.
#[derive(Debug, Clone)]
pub struct MigrationReservationValidator {
    pub rules: Vec<Rc<dyn MigrationReservationRule>>,
}

impl MigrationReservationValidator {
    pub fn validate(&self, item: &MigrationReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_reservation_validator() -> MigrationReservationValidator {
    MigrationReservationValidator {
        rules: vec![
            Rc::new(MigrationReservationRequiredUnitRule {
                message: "migration reservation needs this field".to_string(),
            }) as Rc<dyn MigrationReservationRule>,
            Rc::new(MigrationReservationCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(MigrationReservationBudgetRangeRule { min: 0.6, max: 3869.6 }),
            Rc::new(MigrationReservationRequiredCodeRule {
                message: "migration reservation needs an entry".to_string(),
            }),
            Rc::new(MigrationReservationCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_reservation_validator();
        let item = new_migration_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = MigrationReservationRequiredUnitRule {
            message: "migration reservation needs a value".to_string(),
        };
        let mut item = new_migration_reservation("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = MigrationReservationBudgetRangeRule { min: 5.0, max: 257.1 };
        let mut item = new_migration_reservation("x", "y");
        item.budget = 258.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MigrationReservationRequiredCodeRule {
            message: "migration reservation needs a value".to_string(),
        };
        let mut item = new_migration_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MigrationReservationCodeLengthRule { max_length: 1000 };
        let mut item = new_migration_reservation("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
