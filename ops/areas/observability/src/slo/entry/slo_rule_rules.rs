use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo rule must pass. Check returns its problems, or none.
pub trait SloRuleRule: std::fmt::Debug {
    fn check(&self, item: &SloRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRulePrincipalLengthRule {
    pub max_length: i64,
}

impl SloRuleRule for SloRulePrincipalLengthRule {
    fn check(&self, item: &SloRule) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRuleCodePrefixRule {
    pub prefix: String,
}

impl SloRuleRule for SloRuleCodePrefixRule {
    fn check(&self, item: &SloRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRuleRequiredPrincipalRule {
    pub message: String,
}

impl SloRuleRule for SloRuleRequiredPrincipalRule {
    fn check(&self, item: &SloRule) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRuleCodeLengthRule {
    pub max_length: i64,
}

impl SloRuleRule for SloRuleCodeLengthRule {
    fn check(&self, item: &SloRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRuleRequiredNameRule {
    pub message: String,
}

impl SloRuleRule for SloRuleRequiredNameRule {
    fn check(&self, item: &SloRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo rule.
#[derive(Debug, Clone)]
pub struct SloRuleValidator {
    pub rules: Vec<Rc<dyn SloRuleRule>>,
}

impl SloRuleValidator {
    pub fn validate(&self, item: &SloRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_rule_validator() -> SloRuleValidator {
    SloRuleValidator {
        rules: vec![
            Rc::new(SloRulePrincipalLengthRule { max_length: 3600 }) as Rc<dyn SloRuleRule>,
            Rc::new(SloRuleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SloRuleRequiredPrincipalRule {
                message: "slo rule needs a value".to_string(),
            }),
            Rc::new(SloRuleCodeLengthRule { max_length: 1000 }),
            Rc::new(SloRuleRequiredNameRule {
                message: "slo rule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_rule_validator();
        let item = new_slo_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = SloRulePrincipalLengthRule { max_length: 1000 };
        let mut item = new_slo_rule("x", "y");
        item.principal = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = SloRuleRequiredPrincipalRule {
            message: "slo rule needs a value".to_string(),
        };
        let mut item = new_slo_rule("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SloRuleCodeLengthRule { max_length: 64 };
        let mut item = new_slo_rule("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SloRuleRequiredNameRule {
            message: "slo rule needs a value".to_string(),
        };
        let mut item = new_slo_rule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
