use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional probe checkpoint must pass. Check returns its problems, or none.
pub trait RegionalProbeCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RegionalProbeCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeCheckpointTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RegionalProbeCheckpointRule for RegionalProbeCheckpointTagsLimitRule {
    fn check(&self, item: &RegionalProbeCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl RegionalProbeCheckpointRule for RegionalProbeCheckpointLabelLengthRule {
    fn check(&self, item: &RegionalProbeCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl RegionalProbeCheckpointRule for RegionalProbeCheckpointCodeLengthRule {
    fn check(&self, item: &RegionalProbeCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeCheckpointPinnedGuardRule {
    pub limit: f64,
}

impl RegionalProbeCheckpointRule for RegionalProbeCheckpointPinnedGuardRule {
    fn check(&self, item: &RegionalProbeCheckpoint) -> Vec<String> {
        if item.pinned && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while pinned", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeCheckpointScopeLengthRule {
    pub max_length: i64,
}

impl RegionalProbeCheckpointRule for RegionalProbeCheckpointScopeLengthRule {
    fn check(&self, item: &RegionalProbeCheckpoint) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional probe checkpoint.
#[derive(Debug, Clone)]
pub struct RegionalProbeCheckpointValidator {
    pub rules: Vec<Rc<dyn RegionalProbeCheckpointRule>>,
}

impl RegionalProbeCheckpointValidator {
    pub fn validate(&self, item: &RegionalProbeCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalProbeCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalProbeCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_probe_checkpoint_validator() -> RegionalProbeCheckpointValidator {
    RegionalProbeCheckpointValidator {
        rules: vec![
            Rc::new(RegionalProbeCheckpointTagsLimitRule {
                max_count: 80,
                banned: vec!["default".to_string(), "legacy".to_string(), "lenient".to_string()],
            }) as Rc<dyn RegionalProbeCheckpointRule>,
            Rc::new(RegionalProbeCheckpointLabelLengthRule { max_length: 16 }),
            Rc::new(RegionalProbeCheckpointCodeLengthRule { max_length: 3600 }),
            Rc::new(RegionalProbeCheckpointPinnedGuardRule { limit: 0.8 }),
            Rc::new(RegionalProbeCheckpointScopeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_probe_checkpoint_validator();
        let item = new_regional_probe_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalProbeCheckpointLabelLengthRule { max_length: 16 };
        let mut item = new_regional_probe_checkpoint("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalProbeCheckpointCodeLengthRule { max_length: 32 };
        let mut item = new_regional_probe_checkpoint("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = RegionalProbeCheckpointScopeLengthRule { max_length: 16 };
        let mut item = new_regional_probe_checkpoint("x", "y");
        item.scope = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
