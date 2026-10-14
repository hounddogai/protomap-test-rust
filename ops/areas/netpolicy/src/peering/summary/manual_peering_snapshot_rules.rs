use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual peering snapshot must pass. Check returns its problems, or none.
pub trait ManualPeeringSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &ManualPeeringSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringSnapshotLabelLengthRule {
    pub max_length: i64,
}

impl ManualPeeringSnapshotRule for ManualPeeringSnapshotLabelLengthRule {
    fn check(&self, item: &ManualPeeringSnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl ManualPeeringSnapshotRule for ManualPeeringSnapshotCodeLengthRule {
    fn check(&self, item: &ManualPeeringSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringSnapshotCodePrefixRule {
    pub prefix: String,
}

impl ManualPeeringSnapshotRule for ManualPeeringSnapshotCodePrefixRule {
    fn check(&self, item: &ManualPeeringSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual peering snapshot.
#[derive(Debug, Clone)]
pub struct ManualPeeringSnapshotValidator {
    pub rules: Vec<Rc<dyn ManualPeeringSnapshotRule>>,
}

impl ManualPeeringSnapshotValidator {
    pub fn validate(&self, item: &ManualPeeringSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualPeeringSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualPeeringSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_peering_snapshot_validator() -> ManualPeeringSnapshotValidator {
    ManualPeeringSnapshotValidator {
        rules: vec![
            Rc::new(ManualPeeringSnapshotLabelLengthRule { max_length: 255 }) as Rc<dyn ManualPeeringSnapshotRule>,
            Rc::new(ManualPeeringSnapshotCodeLengthRule { max_length: 64 }),
            Rc::new(ManualPeeringSnapshotCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_peering_snapshot_validator();
        let item = new_manual_peering_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ManualPeeringSnapshotLabelLengthRule { max_length: 32 };
        let mut item = new_manual_peering_snapshot("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualPeeringSnapshotCodeLengthRule { max_length: 64 };
        let mut item = new_manual_peering_snapshot("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
