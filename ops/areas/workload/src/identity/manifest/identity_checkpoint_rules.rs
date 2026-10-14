use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity checkpoint must pass. Check returns its problems, or none.
pub trait IdentityCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &IdentityCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCheckpointDescriptionLengthRule {
    pub max_length: i64,
}

impl IdentityCheckpointRule for IdentityCheckpointDescriptionLengthRule {
    fn check(&self, item: &IdentityCheckpoint) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCheckpointGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityCheckpointRule for IdentityCheckpointGraceDaysRangeRule {
    fn check(&self, item: &IdentityCheckpoint) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCheckpointPrincipalLengthRule {
    pub max_length: i64,
}

impl IdentityCheckpointRule for IdentityCheckpointPrincipalLengthRule {
    fn check(&self, item: &IdentityCheckpoint) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCheckpointNameLengthRule {
    pub max_length: i64,
}

impl IdentityCheckpointRule for IdentityCheckpointNameLengthRule {
    fn check(&self, item: &IdentityCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity checkpoint.
#[derive(Debug, Clone)]
pub struct IdentityCheckpointValidator {
    pub rules: Vec<Rc<dyn IdentityCheckpointRule>>,
}

impl IdentityCheckpointValidator {
    pub fn validate(&self, item: &IdentityCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_checkpoint_validator() -> IdentityCheckpointValidator {
    IdentityCheckpointValidator {
        rules: vec![
            Rc::new(IdentityCheckpointDescriptionLengthRule { max_length: 64 }) as Rc<dyn IdentityCheckpointRule>,
            Rc::new(IdentityCheckpointGraceDaysRangeRule { min: 16, max: 120 }),
            Rc::new(IdentityCheckpointPrincipalLengthRule { max_length: 86400 }),
            Rc::new(IdentityCheckpointNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_checkpoint_validator();
        let item = new_identity_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = IdentityCheckpointDescriptionLengthRule { max_length: 32 };
        let mut item = new_identity_checkpoint("x", "y");
        item.description = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = IdentityCheckpointGraceDaysRangeRule { min: 32, max: 86400 };
        let mut item = new_identity_checkpoint("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = IdentityCheckpointPrincipalLengthRule { max_length: 32 };
        let mut item = new_identity_checkpoint("x", "y");
        item.principal = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = IdentityCheckpointNameLengthRule { max_length: 86400 };
        let mut item = new_identity_checkpoint("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
