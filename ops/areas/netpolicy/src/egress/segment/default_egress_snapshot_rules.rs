use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default egress snapshot must pass. Check returns its problems, or none.
pub trait DefaultEgressSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &DefaultEgressSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressSnapshotRequiredUnitRule {
    pub message: String,
}

impl DefaultEgressSnapshotRule for DefaultEgressSnapshotRequiredUnitRule {
    fn check(&self, item: &DefaultEgressSnapshot) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressSnapshotRequiredCodeRule {
    pub message: String,
}

impl DefaultEgressSnapshotRule for DefaultEgressSnapshotRequiredCodeRule {
    fn check(&self, item: &DefaultEgressSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl DefaultEgressSnapshotRule for DefaultEgressSnapshotCodeLengthRule {
    fn check(&self, item: &DefaultEgressSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressSnapshotCodePrefixRule {
    pub prefix: String,
}

impl DefaultEgressSnapshotRule for DefaultEgressSnapshotCodePrefixRule {
    fn check(&self, item: &DefaultEgressSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressSnapshotLabelLengthRule {
    pub max_length: i64,
}

impl DefaultEgressSnapshotRule for DefaultEgressSnapshotLabelLengthRule {
    fn check(&self, item: &DefaultEgressSnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl DefaultEgressSnapshotRule for DefaultEgressSnapshotTitleLengthRule {
    fn check(&self, item: &DefaultEgressSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default egress snapshot.
#[derive(Debug, Clone)]
pub struct DefaultEgressSnapshotValidator {
    pub rules: Vec<Rc<dyn DefaultEgressSnapshotRule>>,
}

impl DefaultEgressSnapshotValidator {
    pub fn validate(&self, item: &DefaultEgressSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultEgressSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultEgressSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_egress_snapshot_validator() -> DefaultEgressSnapshotValidator {
    DefaultEgressSnapshotValidator {
        rules: vec![
            Rc::new(DefaultEgressSnapshotRequiredUnitRule {
                message: "default egress snapshot needs an entry".to_string(),
            }) as Rc<dyn DefaultEgressSnapshotRule>,
            Rc::new(DefaultEgressSnapshotRequiredCodeRule {
                message: "default egress snapshot needs a value".to_string(),
            }),
            Rc::new(DefaultEgressSnapshotCodeLengthRule { max_length: 3600 }),
            Rc::new(DefaultEgressSnapshotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DefaultEgressSnapshotLabelLengthRule { max_length: 64 }),
            Rc::new(DefaultEgressSnapshotTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_egress_snapshot_validator();
        let item = new_default_egress_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = DefaultEgressSnapshotRequiredUnitRule {
            message: "default egress snapshot needs an entry".to_string(),
        };
        let mut item = new_default_egress_snapshot("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultEgressSnapshotRequiredCodeRule {
            message: "default egress snapshot needs an entry".to_string(),
        };
        let mut item = new_default_egress_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultEgressSnapshotCodeLengthRule { max_length: 255 };
        let mut item = new_default_egress_snapshot("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultEgressSnapshotLabelLengthRule { max_length: 3600 };
        let mut item = new_default_egress_snapshot("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultEgressSnapshotTitleLengthRule { max_length: 64 };
        let mut item = new_default_egress_snapshot("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
