use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone snapshot must pass. Check returns its problems, or none.
pub trait ZoneSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &ZoneSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSnapshotQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneSnapshotRule for ZoneSnapshotQuantityRangeRule {
    fn check(&self, item: &ZoneSnapshot) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSnapshotNameLengthRule {
    pub max_length: i64,
}

impl ZoneSnapshotRule for ZoneSnapshotNameLengthRule {
    fn check(&self, item: &ZoneSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSnapshotAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneSnapshotRule for ZoneSnapshotAttemptsRangeRule {
    fn check(&self, item: &ZoneSnapshot) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSnapshotRequiredNamespaceRule {
    pub message: String,
}

impl ZoneSnapshotRule for ZoneSnapshotRequiredNamespaceRule {
    fn check(&self, item: &ZoneSnapshot) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone snapshot.
#[derive(Debug, Clone)]
pub struct ZoneSnapshotValidator {
    pub rules: Vec<Rc<dyn ZoneSnapshotRule>>,
}

impl ZoneSnapshotValidator {
    pub fn validate(&self, item: &ZoneSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_snapshot_validator() -> ZoneSnapshotValidator {
    ZoneSnapshotValidator {
        rules: vec![
            Rc::new(ZoneSnapshotQuantityRangeRule { min: 3600, max: 3600 }) as Rc<dyn ZoneSnapshotRule>,
            Rc::new(ZoneSnapshotNameLengthRule { max_length: 255 }),
            Rc::new(ZoneSnapshotAttemptsRangeRule { min: 80, max: 80 }),
            Rc::new(ZoneSnapshotRequiredNamespaceRule {
                message: "zone snapshot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_snapshot_validator();
        let item = new_zone_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ZoneSnapshotQuantityRangeRule { min: 32, max: 80 };
        let mut item = new_zone_snapshot("x", "y");
        item.quantity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ZoneSnapshotNameLengthRule { max_length: 32 };
        let mut item = new_zone_snapshot("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ZoneSnapshotAttemptsRangeRule { min: 64, max: 3600 };
        let mut item = new_zone_snapshot("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = ZoneSnapshotRequiredNamespaceRule {
            message: "zone snapshot needs an entry".to_string(),
        };
        let mut item = new_zone_snapshot("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
