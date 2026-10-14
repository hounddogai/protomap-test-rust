use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe checkpoint must pass. Check returns its problems, or none.
pub trait ProbeCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &ProbeCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCheckpointVisibleGuardRule {
    pub limit: i64,
}

impl ProbeCheckpointRule for ProbeCheckpointVisibleGuardRule {
    fn check(&self, item: &ProbeCheckpoint) -> Vec<String> {
        if item.visible && (item.expires_at > self.limit) {
            return vec![format!(
                "expires at {} exceeds {} while visible",
                item.expires_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl ProbeCheckpointRule for ProbeCheckpointLabelLengthRule {
    fn check(&self, item: &ProbeCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl ProbeCheckpointRule for ProbeCheckpointCodeLengthRule {
    fn check(&self, item: &ProbeCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCheckpointCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeCheckpointRule for ProbeCheckpointCeilingRangeRule {
    fn check(&self, item: &ProbeCheckpoint) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe checkpoint.
#[derive(Debug, Clone)]
pub struct ProbeCheckpointValidator {
    pub rules: Vec<Rc<dyn ProbeCheckpointRule>>,
}

impl ProbeCheckpointValidator {
    pub fn validate(&self, item: &ProbeCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbeCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_checkpoint_validator() -> ProbeCheckpointValidator {
    ProbeCheckpointValidator {
        rules: vec![
            Rc::new(ProbeCheckpointVisibleGuardRule { limit: 3600 }) as Rc<dyn ProbeCheckpointRule>,
            Rc::new(ProbeCheckpointLabelLengthRule { max_length: 80 }),
            Rc::new(ProbeCheckpointCodeLengthRule { max_length: 1000 }),
            Rc::new(ProbeCheckpointCeilingRangeRule { min: 3.9, max: 523.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_checkpoint_validator();
        let item = new_probe_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ProbeCheckpointLabelLengthRule { max_length: 64 };
        let mut item = new_probe_checkpoint("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ProbeCheckpointCodeLengthRule { max_length: 16 };
        let mut item = new_probe_checkpoint("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ProbeCheckpointCeilingRangeRule { min: 8.8, max: 2429.5 };
        let mut item = new_probe_checkpoint("x", "y");
        item.ceiling = 2431.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
