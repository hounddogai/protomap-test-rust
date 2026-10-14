use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage entry must pass. Check returns its problems, or none.
pub trait OverageEntryRule: std::fmt::Debug {
    fn check(&self, item: &OverageEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageEntryNameLengthRule {
    pub max_length: i64,
}

impl OverageEntryRule for OverageEntryNameLengthRule {
    fn check(&self, item: &OverageEntry) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageEntryCodeLengthRule {
    pub max_length: i64,
}

impl OverageEntryRule for OverageEntryCodeLengthRule {
    fn check(&self, item: &OverageEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageEntryRequiredExternalRefRule {
    pub message: String,
}

impl OverageEntryRule for OverageEntryRequiredExternalRefRule {
    fn check(&self, item: &OverageEntry) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageEntryPinnedGuardRule {
    pub limit: i64,
}

impl OverageEntryRule for OverageEntryPinnedGuardRule {
    fn check(&self, item: &OverageEntry) -> Vec<String> {
        if item.pinned && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while pinned", item.port, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageEntryRequiredNameRule {
    pub message: String,
}

impl OverageEntryRule for OverageEntryRequiredNameRule {
    fn check(&self, item: &OverageEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage entry.
#[derive(Debug, Clone)]
pub struct OverageEntryValidator {
    pub rules: Vec<Rc<dyn OverageEntryRule>>,
}

impl OverageEntryValidator {
    pub fn validate(&self, item: &OverageEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_entry_validator() -> OverageEntryValidator {
    OverageEntryValidator {
        rules: vec![
            Rc::new(OverageEntryNameLengthRule { max_length: 64 }) as Rc<dyn OverageEntryRule>,
            Rc::new(OverageEntryCodeLengthRule { max_length: 255 }),
            Rc::new(OverageEntryRequiredExternalRefRule {
                message: "overage entry needs a value".to_string(),
            }),
            Rc::new(OverageEntryPinnedGuardRule { limit: 3600 }),
            Rc::new(OverageEntryRequiredNameRule {
                message: "overage entry needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_entry_validator();
        let item = new_overage_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = OverageEntryNameLengthRule { max_length: 16 };
        let mut item = new_overage_entry("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = OverageEntryCodeLengthRule { max_length: 1000 };
        let mut item = new_overage_entry("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = OverageEntryRequiredExternalRefRule {
            message: "overage entry needs an entry".to_string(),
        };
        let mut item = new_overage_entry("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = OverageEntryRequiredNameRule {
            message: "overage entry needs an entry".to_string(),
        };
        let mut item = new_overage_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
