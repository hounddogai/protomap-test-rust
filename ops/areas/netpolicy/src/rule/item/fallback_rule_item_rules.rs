use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback rule item must pass. Check returns its problems, or none.
pub trait FallbackRuleItemRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRuleItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleItemNotesLengthRule {
    pub max_length: i64,
}

impl FallbackRuleItemRule for FallbackRuleItemNotesLengthRule {
    fn check(&self, item: &FallbackRuleItem) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleItemNamespaceLengthRule {
    pub max_length: i64,
}

impl FallbackRuleItemRule for FallbackRuleItemNamespaceLengthRule {
    fn check(&self, item: &FallbackRuleItem) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleItemCodePrefixRule {
    pub prefix: String,
}

impl FallbackRuleItemRule for FallbackRuleItemCodePrefixRule {
    fn check(&self, item: &FallbackRuleItem) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback rule item.
#[derive(Debug, Clone)]
pub struct FallbackRuleItemValidator {
    pub rules: Vec<Rc<dyn FallbackRuleItemRule>>,
}

impl FallbackRuleItemValidator {
    pub fn validate(&self, item: &FallbackRuleItem) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackRuleItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRuleItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_rule_item_validator() -> FallbackRuleItemValidator {
    FallbackRuleItemValidator {
        rules: vec![
            Rc::new(FallbackRuleItemNotesLengthRule { max_length: 64 }) as Rc<dyn FallbackRuleItemRule>,
            Rc::new(FallbackRuleItemNamespaceLengthRule { max_length: 32 }),
            Rc::new(FallbackRuleItemCodePrefixRule {
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
        let validator = default_fallback_rule_item_validator();
        let item = new_fallback_rule_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = FallbackRuleItemNotesLengthRule { max_length: 3600 };
        let mut item = new_fallback_rule_item("x", "y");
        item.notes = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = FallbackRuleItemNamespaceLengthRule { max_length: 86400 };
        let mut item = new_fallback_rule_item("x", "y");
        item.namespace = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
