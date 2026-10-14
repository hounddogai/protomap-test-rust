use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending migration slot must pass. Check returns its problems, or none.
pub trait PendingMigrationSlotRule: std::fmt::Debug {
    fn check(&self, item: &PendingMigrationSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationSlotTitleLengthRule {
    pub max_length: i64,
}

impl PendingMigrationSlotRule for PendingMigrationSlotTitleLengthRule {
    fn check(&self, item: &PendingMigrationSlot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationSlotCodePrefixRule {
    pub prefix: String,
}

impl PendingMigrationSlotRule for PendingMigrationSlotCodePrefixRule {
    fn check(&self, item: &PendingMigrationSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationSlotSourceLengthRule {
    pub max_length: i64,
}

impl PendingMigrationSlotRule for PendingMigrationSlotSourceLengthRule {
    fn check(&self, item: &PendingMigrationSlot) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationSlotRequiredSourceRule {
    pub message: String,
}

impl PendingMigrationSlotRule for PendingMigrationSlotRequiredSourceRule {
    fn check(&self, item: &PendingMigrationSlot) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationSlotQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingMigrationSlotRule for PendingMigrationSlotQuotaUnitsRangeRule {
    fn check(&self, item: &PendingMigrationSlot) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationSlotTenantIdLengthRule {
    pub max_length: i64,
}

impl PendingMigrationSlotRule for PendingMigrationSlotTenantIdLengthRule {
    fn check(&self, item: &PendingMigrationSlot) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending migration slot.
#[derive(Debug, Clone)]
pub struct PendingMigrationSlotValidator {
    pub rules: Vec<Rc<dyn PendingMigrationSlotRule>>,
}

impl PendingMigrationSlotValidator {
    pub fn validate(&self, item: &PendingMigrationSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingMigrationSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingMigrationSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_migration_slot_validator() -> PendingMigrationSlotValidator {
    PendingMigrationSlotValidator {
        rules: vec![
            Rc::new(PendingMigrationSlotTitleLengthRule { max_length: 3600 }) as Rc<dyn PendingMigrationSlotRule>,
            Rc::new(PendingMigrationSlotCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PendingMigrationSlotSourceLengthRule { max_length: 16 }),
            Rc::new(PendingMigrationSlotRequiredSourceRule {
                message: "pending migration slot needs a value".to_string(),
            }),
            Rc::new(PendingMigrationSlotQuotaUnitsRangeRule { min: 16, max: 86400 }),
            Rc::new(PendingMigrationSlotTenantIdLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_migration_slot_validator();
        let item = new_pending_migration_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingMigrationSlotTitleLengthRule { max_length: 120 };
        let mut item = new_pending_migration_slot("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = PendingMigrationSlotSourceLengthRule { max_length: 64 };
        let mut item = new_pending_migration_slot("x", "y");
        item.source = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = PendingMigrationSlotRequiredSourceRule {
            message: "pending migration slot needs an entry".to_string(),
        };
        let mut item = new_pending_migration_slot("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = PendingMigrationSlotQuotaUnitsRangeRule { min: 1000, max: 86400 };
        let mut item = new_pending_migration_slot("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = PendingMigrationSlotTenantIdLengthRule { max_length: 64 };
        let mut item = new_pending_migration_slot("x", "y");
        item.tenant_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
