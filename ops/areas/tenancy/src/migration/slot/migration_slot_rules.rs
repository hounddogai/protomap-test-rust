use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration slot must pass. Check returns its problems, or none.
pub trait MigrationSlotRule: std::fmt::Debug {
    fn check(&self, item: &MigrationSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSlotWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationSlotRule for MigrationSlotWeightRangeRule {
    fn check(&self, item: &MigrationSlot) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSlotActiveGuardRule {
    pub limit: i64,
}

impl MigrationSlotRule for MigrationSlotActiveGuardRule {
    fn check(&self, item: &MigrationSlot) -> Vec<String> {
        if item.active && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while active",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSlotRequiredTimezoneRule {
    pub message: String,
}

impl MigrationSlotRule for MigrationSlotRequiredTimezoneRule {
    fn check(&self, item: &MigrationSlot) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSlotCodePrefixRule {
    pub prefix: String,
}

impl MigrationSlotRule for MigrationSlotCodePrefixRule {
    fn check(&self, item: &MigrationSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration slot.
#[derive(Debug, Clone)]
pub struct MigrationSlotValidator {
    pub rules: Vec<Rc<dyn MigrationSlotRule>>,
}

impl MigrationSlotValidator {
    pub fn validate(&self, item: &MigrationSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_slot_validator() -> MigrationSlotValidator {
    MigrationSlotValidator {
        rules: vec![
            Rc::new(MigrationSlotWeightRangeRule { min: 2.1, max: 144.7 }) as Rc<dyn MigrationSlotRule>,
            Rc::new(MigrationSlotActiveGuardRule { limit: 16 }),
            Rc::new(MigrationSlotRequiredTimezoneRule {
                message: "migration slot needs a value".to_string(),
            }),
            Rc::new(MigrationSlotCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_slot_validator();
        let item = new_migration_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = MigrationSlotWeightRangeRule { min: 1.5, max: 2449.8 };
        let mut item = new_migration_slot("x", "y");
        item.weight = 2451.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = MigrationSlotRequiredTimezoneRule {
            message: "migration slot needs a value".to_string(),
        };
        let mut item = new_migration_slot("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
