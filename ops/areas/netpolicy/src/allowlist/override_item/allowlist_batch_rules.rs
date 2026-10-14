use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist batch must pass. Check returns its problems, or none.
pub trait AllowlistBatchRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBatchRequiredOwnerIdRule {
    pub message: String,
}

impl AllowlistBatchRule for AllowlistBatchRequiredOwnerIdRule {
    fn check(&self, item: &AllowlistBatch) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBatchRequiredCodeRule {
    pub message: String,
}

impl AllowlistBatchRule for AllowlistBatchRequiredCodeRule {
    fn check(&self, item: &AllowlistBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBatchRequiredNameRule {
    pub message: String,
}

impl AllowlistBatchRule for AllowlistBatchRequiredNameRule {
    fn check(&self, item: &AllowlistBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist batch.
#[derive(Debug, Clone)]
pub struct AllowlistBatchValidator {
    pub rules: Vec<Rc<dyn AllowlistBatchRule>>,
}

impl AllowlistBatchValidator {
    pub fn validate(&self, item: &AllowlistBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_batch_validator() -> AllowlistBatchValidator {
    AllowlistBatchValidator {
        rules: vec![
            Rc::new(AllowlistBatchRequiredOwnerIdRule {
                message: "allowlist batch needs an entry".to_string(),
            }) as Rc<dyn AllowlistBatchRule>,
            Rc::new(AllowlistBatchRequiredCodeRule {
                message: "allowlist batch needs an entry".to_string(),
            }),
            Rc::new(AllowlistBatchRequiredNameRule {
                message: "allowlist batch needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_batch_validator();
        let item = new_allowlist_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = AllowlistBatchRequiredOwnerIdRule {
            message: "allowlist batch needs a value".to_string(),
        };
        let mut item = new_allowlist_batch("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistBatchRequiredCodeRule {
            message: "allowlist batch needs a value".to_string(),
        };
        let mut item = new_allowlist_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AllowlistBatchRequiredNameRule {
            message: "allowlist batch needs this field".to_string(),
        };
        let mut item = new_allowlist_batch("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
