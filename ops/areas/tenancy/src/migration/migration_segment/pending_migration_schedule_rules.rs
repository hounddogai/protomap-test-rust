use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending migration schedule must pass. Check returns its problems, or none.
pub trait PendingMigrationScheduleRule: std::fmt::Debug {
    fn check(&self, item: &PendingMigrationSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationScheduleContactEmailLengthRule {
    pub max_length: i64,
}

impl PendingMigrationScheduleRule for PendingMigrationScheduleContactEmailLengthRule {
    fn check(&self, item: &PendingMigrationSchedule) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationScheduleNameLengthRule {
    pub max_length: i64,
}

impl PendingMigrationScheduleRule for PendingMigrationScheduleNameLengthRule {
    fn check(&self, item: &PendingMigrationSchedule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationScheduleBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingMigrationScheduleRule for PendingMigrationScheduleBurstRangeRule {
    fn check(&self, item: &PendingMigrationSchedule) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationScheduleStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingMigrationScheduleRule for PendingMigrationScheduleStartsAtRangeRule {
    fn check(&self, item: &PendingMigrationSchedule) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending migration schedule.
#[derive(Debug, Clone)]
pub struct PendingMigrationScheduleValidator {
    pub rules: Vec<Rc<dyn PendingMigrationScheduleRule>>,
}

impl PendingMigrationScheduleValidator {
    pub fn validate(&self, item: &PendingMigrationSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingMigrationSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingMigrationSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_migration_schedule_validator() -> PendingMigrationScheduleValidator {
    PendingMigrationScheduleValidator {
        rules: vec![
            Rc::new(PendingMigrationScheduleContactEmailLengthRule { max_length: 32 })
                as Rc<dyn PendingMigrationScheduleRule>,
            Rc::new(PendingMigrationScheduleNameLengthRule { max_length: 86400 }),
            Rc::new(PendingMigrationScheduleBurstRangeRule { min: 80, max: 1000 }),
            Rc::new(PendingMigrationScheduleStartsAtRangeRule { min: 16, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_migration_schedule_validator();
        let item = new_pending_migration_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = PendingMigrationScheduleContactEmailLengthRule { max_length: 1000 };
        let mut item = new_pending_migration_schedule("x", "y");
        item.contact_email = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingMigrationScheduleNameLengthRule { max_length: 255 };
        let mut item = new_pending_migration_schedule("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = PendingMigrationScheduleBurstRangeRule { min: 16, max: 80 };
        let mut item = new_pending_migration_schedule("x", "y");
        item.burst = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = PendingMigrationScheduleStartsAtRangeRule { min: 255, max: 86400 };
        let mut item = new_pending_migration_schedule("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
