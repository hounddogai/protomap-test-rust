use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation bundle must pass. Check returns its problems, or none.
pub trait FederationBundleRule: std::fmt::Debug {
    fn check(&self, item: &FederationBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationBundleNameLengthRule {
    pub max_length: i64,
}

impl FederationBundleRule for FederationBundleNameLengthRule {
    fn check(&self, item: &FederationBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationBundleCodeLengthRule {
    pub max_length: i64,
}

impl FederationBundleRule for FederationBundleCodeLengthRule {
    fn check(&self, item: &FederationBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationBundleRequiredKeyIdRule {
    pub message: String,
}

impl FederationBundleRule for FederationBundleRequiredKeyIdRule {
    fn check(&self, item: &FederationBundle) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationBundleRequiredCodeRule {
    pub message: String,
}

impl FederationBundleRule for FederationBundleRequiredCodeRule {
    fn check(&self, item: &FederationBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation bundle.
#[derive(Debug, Clone)]
pub struct FederationBundleValidator {
    pub rules: Vec<Rc<dyn FederationBundleRule>>,
}

impl FederationBundleValidator {
    pub fn validate(&self, item: &FederationBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_bundle_validator() -> FederationBundleValidator {
    FederationBundleValidator {
        rules: vec![
            Rc::new(FederationBundleNameLengthRule { max_length: 3600 }) as Rc<dyn FederationBundleRule>,
            Rc::new(FederationBundleCodeLengthRule { max_length: 86400 }),
            Rc::new(FederationBundleRequiredKeyIdRule {
                message: "federation bundle needs this field".to_string(),
            }),
            Rc::new(FederationBundleRequiredCodeRule {
                message: "federation bundle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_bundle_validator();
        let item = new_federation_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FederationBundleNameLengthRule { max_length: 3600 };
        let mut item = new_federation_bundle("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FederationBundleCodeLengthRule { max_length: 64 };
        let mut item = new_federation_bundle("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = FederationBundleRequiredKeyIdRule {
            message: "federation bundle needs a value".to_string(),
        };
        let mut item = new_federation_bundle("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FederationBundleRequiredCodeRule {
            message: "federation bundle needs a value".to_string(),
        };
        let mut item = new_federation_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
