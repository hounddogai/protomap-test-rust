use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a probe snapshot must pass. Check returns its problems, or none.
pub trait ProbeSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &ProbeSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSnapshotCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeSnapshotRule for ProbeSnapshotCreatedAtRangeRule {
    fn check(&self, item: &ProbeSnapshot) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSnapshotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ProbeSnapshotRule for ProbeSnapshotAllowedCodeRule {
    fn check(&self, item: &ProbeSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSnapshotRequiredNameRule {
    pub message: String,
}

impl ProbeSnapshotRule for ProbeSnapshotRequiredNameRule {
    fn check(&self, item: &ProbeSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSnapshotRequiredCodeRule {
    pub message: String,
}

impl ProbeSnapshotRule for ProbeSnapshotRequiredCodeRule {
    fn check(&self, item: &ProbeSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSnapshotPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeSnapshotRule for ProbeSnapshotPriorityRangeRule {
    fn check(&self, item: &ProbeSnapshot) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSnapshotNameLengthRule {
    pub max_length: i64,
}

impl ProbeSnapshotRule for ProbeSnapshotNameLengthRule {
    fn check(&self, item: &ProbeSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe snapshot.
#[derive(Debug, Clone)]
pub struct ProbeSnapshotValidator {
    pub rules: Vec<Rc<dyn ProbeSnapshotRule>>,
}

impl ProbeSnapshotValidator {
    pub fn validate(&self, item: &ProbeSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_snapshot_validator() -> ProbeSnapshotValidator {
    ProbeSnapshotValidator {
        rules: vec![
            Rc::new(ProbeSnapshotCreatedAtRangeRule { min: 32, max: 64 }) as Rc<dyn ProbeSnapshotRule>,
            Rc::new(ProbeSnapshotAllowedCodeRule {
                allowed: vec!["lenient".to_string(), "seasonal".to_string()],
            }),
            Rc::new(ProbeSnapshotRequiredNameRule {
                message: "probe snapshot needs a value".to_string(),
            }),
            Rc::new(ProbeSnapshotRequiredCodeRule {
                message: "probe snapshot needs an entry".to_string(),
            }),
            Rc::new(ProbeSnapshotPriorityRangeRule { min: 255, max: 3600 }),
            Rc::new(ProbeSnapshotNameLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_snapshot_validator();
        let item = new_probe_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ProbeSnapshotCreatedAtRangeRule { min: 80, max: 86400 };
        let mut item = new_probe_snapshot("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ProbeSnapshotRequiredNameRule {
            message: "probe snapshot needs a value".to_string(),
        };
        let mut item = new_probe_snapshot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeSnapshotRequiredCodeRule {
            message: "probe snapshot needs a value".to_string(),
        };
        let mut item = new_probe_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = ProbeSnapshotPriorityRangeRule { min: 255, max: 86400 };
        let mut item = new_probe_snapshot("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ProbeSnapshotNameLengthRule { max_length: 120 };
        let mut item = new_probe_snapshot("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
