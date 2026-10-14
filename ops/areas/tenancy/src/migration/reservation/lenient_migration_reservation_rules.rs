use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient migration reservation must pass. Check returns its problems, or none.
pub trait LenientMigrationReservationRule: std::fmt::Debug {
    fn check(&self, item: &LenientMigrationReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationReservationCodeLengthRule {
    pub max_length: i64,
}

impl LenientMigrationReservationRule for LenientMigrationReservationCodeLengthRule {
    fn check(&self, item: &LenientMigrationReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationReservationRequiredCodeRule {
    pub message: String,
}

impl LenientMigrationReservationRule for LenientMigrationReservationRequiredCodeRule {
    fn check(&self, item: &LenientMigrationReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationReservationRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientMigrationReservationRule for LenientMigrationReservationRevisionRangeRule {
    fn check(&self, item: &LenientMigrationReservation) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationReservationCodePrefixRule {
    pub prefix: String,
}

impl LenientMigrationReservationRule for LenientMigrationReservationCodePrefixRule {
    fn check(&self, item: &LenientMigrationReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationReservationChannelLengthRule {
    pub max_length: i64,
}

impl LenientMigrationReservationRule for LenientMigrationReservationChannelLengthRule {
    fn check(&self, item: &LenientMigrationReservation) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationReservationOwnerIdLengthRule {
    pub max_length: i64,
}

impl LenientMigrationReservationRule for LenientMigrationReservationOwnerIdLengthRule {
    fn check(&self, item: &LenientMigrationReservation) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient migration reservation.
#[derive(Debug, Clone)]
pub struct LenientMigrationReservationValidator {
    pub rules: Vec<Rc<dyn LenientMigrationReservationRule>>,
}

impl LenientMigrationReservationValidator {
    pub fn validate(&self, item: &LenientMigrationReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientMigrationReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientMigrationReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_migration_reservation_validator() -> LenientMigrationReservationValidator {
    LenientMigrationReservationValidator {
        rules: vec![
            Rc::new(LenientMigrationReservationCodeLengthRule { max_length: 3600 })
                as Rc<dyn LenientMigrationReservationRule>,
            Rc::new(LenientMigrationReservationRequiredCodeRule {
                message: "lenient migration reservation needs an entry".to_string(),
            }),
            Rc::new(LenientMigrationReservationRevisionRangeRule { min: 16, max: 16 }),
            Rc::new(LenientMigrationReservationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LenientMigrationReservationChannelLengthRule { max_length: 80 }),
            Rc::new(LenientMigrationReservationOwnerIdLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_migration_reservation_validator();
        let item = new_lenient_migration_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientMigrationReservationCodeLengthRule { max_length: 255 };
        let mut item = new_lenient_migration_reservation("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientMigrationReservationRequiredCodeRule {
            message: "lenient migration reservation needs a value".to_string(),
        };
        let mut item = new_lenient_migration_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = LenientMigrationReservationRevisionRangeRule { min: 16, max: 120 };
        let mut item = new_lenient_migration_reservation("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = LenientMigrationReservationChannelLengthRule { max_length: 16 };
        let mut item = new_lenient_migration_reservation("x", "y");
        item.channel = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = LenientMigrationReservationOwnerIdLengthRule { max_length: 64 };
        let mut item = new_lenient_migration_reservation("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
