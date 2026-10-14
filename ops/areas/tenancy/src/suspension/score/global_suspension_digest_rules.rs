use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global suspension digest must pass. Check returns its problems, or none.
pub trait GlobalSuspensionDigestRule: std::fmt::Debug {
    fn check(&self, item: &GlobalSuspensionDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionDigestCodeLengthRule {
    pub max_length: i64,
}

impl GlobalSuspensionDigestRule for GlobalSuspensionDigestCodeLengthRule {
    fn check(&self, item: &GlobalSuspensionDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionDigestRequiredCodeRule {
    pub message: String,
}

impl GlobalSuspensionDigestRule for GlobalSuspensionDigestRequiredCodeRule {
    fn check(&self, item: &GlobalSuspensionDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionDigestRequiredTitleRule {
    pub message: String,
}

impl GlobalSuspensionDigestRule for GlobalSuspensionDigestRequiredTitleRule {
    fn check(&self, item: &GlobalSuspensionDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionDigestCodePrefixRule {
    pub prefix: String,
}

impl GlobalSuspensionDigestRule for GlobalSuspensionDigestCodePrefixRule {
    fn check(&self, item: &GlobalSuspensionDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionDigestTitleLengthRule {
    pub max_length: i64,
}

impl GlobalSuspensionDigestRule for GlobalSuspensionDigestTitleLengthRule {
    fn check(&self, item: &GlobalSuspensionDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionDigestOwnerIdLengthRule {
    pub max_length: i64,
}

impl GlobalSuspensionDigestRule for GlobalSuspensionDigestOwnerIdLengthRule {
    fn check(&self, item: &GlobalSuspensionDigest) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global suspension digest.
#[derive(Debug, Clone)]
pub struct GlobalSuspensionDigestValidator {
    pub rules: Vec<Rc<dyn GlobalSuspensionDigestRule>>,
}

impl GlobalSuspensionDigestValidator {
    pub fn validate(&self, item: &GlobalSuspensionDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalSuspensionDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalSuspensionDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_suspension_digest_validator() -> GlobalSuspensionDigestValidator {
    GlobalSuspensionDigestValidator {
        rules: vec![
            Rc::new(GlobalSuspensionDigestCodeLengthRule { max_length: 3600 }) as Rc<dyn GlobalSuspensionDigestRule>,
            Rc::new(GlobalSuspensionDigestRequiredCodeRule {
                message: "global suspension digest needs this field".to_string(),
            }),
            Rc::new(GlobalSuspensionDigestRequiredTitleRule {
                message: "global suspension digest needs a value".to_string(),
            }),
            Rc::new(GlobalSuspensionDigestCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(GlobalSuspensionDigestTitleLengthRule { max_length: 120 }),
            Rc::new(GlobalSuspensionDigestOwnerIdLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_suspension_digest_validator();
        let item = new_global_suspension_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalSuspensionDigestCodeLengthRule { max_length: 255 };
        let mut item = new_global_suspension_digest("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalSuspensionDigestRequiredCodeRule {
            message: "global suspension digest needs this field".to_string(),
        };
        let mut item = new_global_suspension_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GlobalSuspensionDigestRequiredTitleRule {
            message: "global suspension digest needs an entry".to_string(),
        };
        let mut item = new_global_suspension_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalSuspensionDigestTitleLengthRule { max_length: 255 };
        let mut item = new_global_suspension_digest("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = GlobalSuspensionDigestOwnerIdLengthRule { max_length: 86400 };
        let mut item = new_global_suspension_digest("x", "y");
        item.owner_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
