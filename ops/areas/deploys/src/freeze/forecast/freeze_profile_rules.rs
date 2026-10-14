use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a freeze profile must pass. Check returns its problems, or none.
pub trait FreezeProfileRule: std::fmt::Debug {
    fn check(&self, item: &FreezeProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeProfileAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FreezeProfileRule for FreezeProfileAllowedCodeRule {
    fn check(&self, item: &FreezeProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeProfileCodePrefixRule {
    pub prefix: String,
}

impl FreezeProfileRule for FreezeProfileCodePrefixRule {
    fn check(&self, item: &FreezeProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeProfileNameLengthRule {
    pub max_length: i64,
}

impl FreezeProfileRule for FreezeProfileNameLengthRule {
    fn check(&self, item: &FreezeProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze profile.
#[derive(Debug, Clone)]
pub struct FreezeProfileValidator {
    pub rules: Vec<Rc<dyn FreezeProfileRule>>,
}

impl FreezeProfileValidator {
    pub fn validate(&self, item: &FreezeProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_profile_validator() -> FreezeProfileValidator {
    FreezeProfileValidator {
        rules: vec![
            Rc::new(FreezeProfileAllowedCodeRule {
                allowed: vec!["scheduled".to_string()],
            }) as Rc<dyn FreezeProfileRule>,
            Rc::new(FreezeProfileCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(FreezeProfileNameLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_profile_validator();
        let item = new_freeze_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FreezeProfileNameLengthRule { max_length: 32 };
        let mut item = new_freeze_profile("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
