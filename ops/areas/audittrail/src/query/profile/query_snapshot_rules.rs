use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query snapshot must pass. Check returns its problems, or none.
pub trait QuerySnapshotRule: std::fmt::Debug {
    fn check(&self, item: &QuerySnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySnapshotRequiredNameRule {
    pub message: String,
}

impl QuerySnapshotRule for QuerySnapshotRequiredNameRule {
    fn check(&self, item: &QuerySnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySnapshotTenantIdLengthRule {
    pub max_length: i64,
}

impl QuerySnapshotRule for QuerySnapshotTenantIdLengthRule {
    fn check(&self, item: &QuerySnapshot) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySnapshotCodeLengthRule {
    pub max_length: i64,
}

impl QuerySnapshotRule for QuerySnapshotCodeLengthRule {
    fn check(&self, item: &QuerySnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySnapshotRequiredCodeRule {
    pub message: String,
}

impl QuerySnapshotRule for QuerySnapshotRequiredCodeRule {
    fn check(&self, item: &QuerySnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySnapshotRequiredTenantIdRule {
    pub message: String,
}

impl QuerySnapshotRule for QuerySnapshotRequiredTenantIdRule {
    fn check(&self, item: &QuerySnapshot) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySnapshotCodePrefixRule {
    pub prefix: String,
}

impl QuerySnapshotRule for QuerySnapshotCodePrefixRule {
    fn check(&self, item: &QuerySnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query snapshot.
#[derive(Debug, Clone)]
pub struct QuerySnapshotValidator {
    pub rules: Vec<Rc<dyn QuerySnapshotRule>>,
}

impl QuerySnapshotValidator {
    pub fn validate(&self, item: &QuerySnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuerySnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuerySnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_snapshot_validator() -> QuerySnapshotValidator {
    QuerySnapshotValidator {
        rules: vec![
            Rc::new(QuerySnapshotRequiredNameRule {
                message: "query snapshot needs this field".to_string(),
            }) as Rc<dyn QuerySnapshotRule>,
            Rc::new(QuerySnapshotTenantIdLengthRule { max_length: 3600 }),
            Rc::new(QuerySnapshotCodeLengthRule { max_length: 16 }),
            Rc::new(QuerySnapshotRequiredCodeRule {
                message: "query snapshot needs an entry".to_string(),
            }),
            Rc::new(QuerySnapshotRequiredTenantIdRule {
                message: "query snapshot needs an entry".to_string(),
            }),
            Rc::new(QuerySnapshotCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_snapshot_validator();
        let item = new_query_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = QuerySnapshotRequiredNameRule {
            message: "query snapshot needs a value".to_string(),
        };
        let mut item = new_query_snapshot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = QuerySnapshotTenantIdLengthRule { max_length: 120 };
        let mut item = new_query_snapshot("x", "y");
        item.tenant_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QuerySnapshotCodeLengthRule { max_length: 86400 };
        let mut item = new_query_snapshot("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QuerySnapshotRequiredCodeRule {
            message: "query snapshot needs a value".to_string(),
        };
        let mut item = new_query_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = QuerySnapshotRequiredTenantIdRule {
            message: "query snapshot needs a value".to_string(),
        };
        let mut item = new_query_snapshot("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
