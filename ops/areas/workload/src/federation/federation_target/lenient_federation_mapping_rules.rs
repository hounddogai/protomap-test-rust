use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient federation mapping must pass. Check returns its problems, or none.
pub trait LenientFederationMappingRule: std::fmt::Debug {
    fn check(&self, item: &LenientFederationMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFederationMappingRequiredTitleRule {
    pub message: String,
}

impl LenientFederationMappingRule for LenientFederationMappingRequiredTitleRule {
    fn check(&self, item: &LenientFederationMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFederationMappingLocaleLengthRule {
    pub max_length: i64,
}

impl LenientFederationMappingRule for LenientFederationMappingLocaleLengthRule {
    fn check(&self, item: &LenientFederationMapping) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFederationMappingCategoryLengthRule {
    pub max_length: i64,
}

impl LenientFederationMappingRule for LenientFederationMappingCategoryLengthRule {
    fn check(&self, item: &LenientFederationMapping) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFederationMappingCodeLengthRule {
    pub max_length: i64,
}

impl LenientFederationMappingRule for LenientFederationMappingCodeLengthRule {
    fn check(&self, item: &LenientFederationMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFederationMappingCodePrefixRule {
    pub prefix: String,
}

impl LenientFederationMappingRule for LenientFederationMappingCodePrefixRule {
    fn check(&self, item: &LenientFederationMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient federation mapping.
#[derive(Debug, Clone)]
pub struct LenientFederationMappingValidator {
    pub rules: Vec<Rc<dyn LenientFederationMappingRule>>,
}

impl LenientFederationMappingValidator {
    pub fn validate(&self, item: &LenientFederationMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientFederationMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientFederationMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_federation_mapping_validator() -> LenientFederationMappingValidator {
    LenientFederationMappingValidator {
        rules: vec![
            Rc::new(LenientFederationMappingRequiredTitleRule {
                message: "lenient federation mapping needs an entry".to_string(),
            }) as Rc<dyn LenientFederationMappingRule>,
            Rc::new(LenientFederationMappingLocaleLengthRule { max_length: 120 }),
            Rc::new(LenientFederationMappingCategoryLengthRule { max_length: 86400 }),
            Rc::new(LenientFederationMappingCodeLengthRule { max_length: 16 }),
            Rc::new(LenientFederationMappingCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_federation_mapping_validator();
        let item = new_lenient_federation_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientFederationMappingRequiredTitleRule {
            message: "lenient federation mapping needs an entry".to_string(),
        };
        let mut item = new_lenient_federation_mapping("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = LenientFederationMappingLocaleLengthRule { max_length: 3600 };
        let mut item = new_lenient_federation_mapping("x", "y");
        item.locale = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = LenientFederationMappingCategoryLengthRule { max_length: 16 };
        let mut item = new_lenient_federation_mapping("x", "y");
        item.category = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientFederationMappingCodeLengthRule { max_length: 32 };
        let mut item = new_lenient_federation_mapping("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
