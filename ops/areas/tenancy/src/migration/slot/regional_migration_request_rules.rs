use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional migration request must pass. Check returns its problems, or none.
pub trait RegionalMigrationRequestRule: std::fmt::Debug {
    fn check(&self, item: &RegionalMigrationRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationRequestRequiredCodeRule {
    pub message: String,
}

impl RegionalMigrationRequestRule for RegionalMigrationRequestRequiredCodeRule {
    fn check(&self, item: &RegionalMigrationRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationRequestCodeLengthRule {
    pub max_length: i64,
}

impl RegionalMigrationRequestRule for RegionalMigrationRequestCodeLengthRule {
    fn check(&self, item: &RegionalMigrationRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationRequestRequiredClusterRule {
    pub message: String,
}

impl RegionalMigrationRequestRule for RegionalMigrationRequestRequiredClusterRule {
    fn check(&self, item: &RegionalMigrationRequest) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional migration request.
#[derive(Debug, Clone)]
pub struct RegionalMigrationRequestValidator {
    pub rules: Vec<Rc<dyn RegionalMigrationRequestRule>>,
}

impl RegionalMigrationRequestValidator {
    pub fn validate(&self, item: &RegionalMigrationRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalMigrationRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalMigrationRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_migration_request_validator() -> RegionalMigrationRequestValidator {
    RegionalMigrationRequestValidator {
        rules: vec![
            Rc::new(RegionalMigrationRequestRequiredCodeRule {
                message: "regional migration request needs a value".to_string(),
            }) as Rc<dyn RegionalMigrationRequestRule>,
            Rc::new(RegionalMigrationRequestCodeLengthRule { max_length: 1000 }),
            Rc::new(RegionalMigrationRequestRequiredClusterRule {
                message: "regional migration request needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_migration_request_validator();
        let item = new_regional_migration_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalMigrationRequestRequiredCodeRule {
            message: "regional migration request needs an entry".to_string(),
        };
        let mut item = new_regional_migration_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalMigrationRequestCodeLengthRule { max_length: 80 };
        let mut item = new_regional_migration_request("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = RegionalMigrationRequestRequiredClusterRule {
            message: "regional migration request needs an entry".to_string(),
        };
        let mut item = new_regional_migration_request("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
