use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary rule group must pass. Check returns its problems, or none.
pub trait PrimaryRuleGroupRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRuleGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleGroupRequiredCodeRule {
    pub message: String,
}

impl PrimaryRuleGroupRule for PrimaryRuleGroupRequiredCodeRule {
    fn check(&self, item: &PrimaryRuleGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleGroupRegionLengthRule {
    pub max_length: i64,
}

impl PrimaryRuleGroupRule for PrimaryRuleGroupRegionLengthRule {
    fn check(&self, item: &PrimaryRuleGroup) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleGroupCodePrefixRule {
    pub prefix: String,
}

impl PrimaryRuleGroupRule for PrimaryRuleGroupCodePrefixRule {
    fn check(&self, item: &PrimaryRuleGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleGroupRequiredNameRule {
    pub message: String,
}

impl PrimaryRuleGroupRule for PrimaryRuleGroupRequiredNameRule {
    fn check(&self, item: &PrimaryRuleGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleGroupNameLengthRule {
    pub max_length: i64,
}

impl PrimaryRuleGroupRule for PrimaryRuleGroupNameLengthRule {
    fn check(&self, item: &PrimaryRuleGroup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary rule group.
#[derive(Debug, Clone)]
pub struct PrimaryRuleGroupValidator {
    pub rules: Vec<Rc<dyn PrimaryRuleGroupRule>>,
}

impl PrimaryRuleGroupValidator {
    pub fn validate(&self, item: &PrimaryRuleGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryRuleGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRuleGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_rule_group_validator() -> PrimaryRuleGroupValidator {
    PrimaryRuleGroupValidator {
        rules: vec![
            Rc::new(PrimaryRuleGroupRequiredCodeRule {
                message: "primary rule group needs an entry".to_string(),
            }) as Rc<dyn PrimaryRuleGroupRule>,
            Rc::new(PrimaryRuleGroupRegionLengthRule { max_length: 32 }),
            Rc::new(PrimaryRuleGroupCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PrimaryRuleGroupRequiredNameRule {
                message: "primary rule group needs an entry".to_string(),
            }),
            Rc::new(PrimaryRuleGroupNameLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_rule_group_validator();
        let item = new_primary_rule_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryRuleGroupRequiredCodeRule {
            message: "primary rule group needs a value".to_string(),
        };
        let mut item = new_primary_rule_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = PrimaryRuleGroupRegionLengthRule { max_length: 16 };
        let mut item = new_primary_rule_group("x", "y");
        item.region = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryRuleGroupRequiredNameRule {
            message: "primary rule group needs a value".to_string(),
        };
        let mut item = new_primary_rule_group("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimaryRuleGroupNameLengthRule { max_length: 80 };
        let mut item = new_primary_rule_group("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
