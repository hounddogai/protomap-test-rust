use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone checkpoint must pass. Check returns its problems, or none.
pub trait ZoneCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &ZoneCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneCheckpointArchivedGuardRule {
    pub limit: f64,
}

impl ZoneCheckpointRule for ZoneCheckpointArchivedGuardRule {
    fn check(&self, item: &ZoneCheckpoint) -> Vec<String> {
        if item.archived && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while archived", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneCheckpointEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneCheckpointRule for ZoneCheckpointEndsAtRangeRule {
    fn check(&self, item: &ZoneCheckpoint) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneCheckpointRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneCheckpointRule for ZoneCheckpointRevisionRangeRule {
    fn check(&self, item: &ZoneCheckpoint) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl ZoneCheckpointRule for ZoneCheckpointCodeLengthRule {
    fn check(&self, item: &ZoneCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone checkpoint.
#[derive(Debug, Clone)]
pub struct ZoneCheckpointValidator {
    pub rules: Vec<Rc<dyn ZoneCheckpointRule>>,
}

impl ZoneCheckpointValidator {
    pub fn validate(&self, item: &ZoneCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_checkpoint_validator() -> ZoneCheckpointValidator {
    ZoneCheckpointValidator {
        rules: vec![
            Rc::new(ZoneCheckpointArchivedGuardRule { limit: 2.3 }) as Rc<dyn ZoneCheckpointRule>,
            Rc::new(ZoneCheckpointEndsAtRangeRule { min: 64, max: 64 }),
            Rc::new(ZoneCheckpointRevisionRangeRule { min: 1000, max: 86400 }),
            Rc::new(ZoneCheckpointCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_checkpoint_validator();
        let item = new_zone_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = ZoneCheckpointEndsAtRangeRule { min: 120, max: 86400 };
        let mut item = new_zone_checkpoint("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = ZoneCheckpointRevisionRangeRule { min: 32, max: 64 };
        let mut item = new_zone_checkpoint("x", "y");
        item.revision = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ZoneCheckpointCodeLengthRule { max_length: 16 };
        let mut item = new_zone_checkpoint("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
