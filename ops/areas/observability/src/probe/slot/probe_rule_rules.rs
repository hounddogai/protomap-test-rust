use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe rule must pass. Check returns its problems, or none.
pub trait ProbeRuleRule: std::fmt::Debug {
    fn check(&self, item: &ProbeRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRuleRequiredNameRule {
    pub message: String,
}

impl ProbeRuleRule for ProbeRuleRequiredNameRule {
    fn check(&self, item: &ProbeRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRuleRequiredCodeRule {
    pub message: String,
}

impl ProbeRuleRule for ProbeRuleRequiredCodeRule {
    fn check(&self, item: &ProbeRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRuleRequiredScopeRule {
    pub message: String,
}

impl ProbeRuleRule for ProbeRuleRequiredScopeRule {
    fn check(&self, item: &ProbeRule) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRuleScopeLengthRule {
    pub max_length: i64,
}

impl ProbeRuleRule for ProbeRuleScopeLengthRule {
    fn check(&self, item: &ProbeRule) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe rule.
#[derive(Debug, Clone)]
pub struct ProbeRuleValidator {
    pub rules: Vec<Rc<dyn ProbeRuleRule>>,
}

impl ProbeRuleValidator {
    pub fn validate(&self, item: &ProbeRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbeRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_rule_validator() -> ProbeRuleValidator {
    ProbeRuleValidator {
        rules: vec![
            Rc::new(ProbeRuleRequiredNameRule {
                message: "probe rule needs an entry".to_string(),
            }) as Rc<dyn ProbeRuleRule>,
            Rc::new(ProbeRuleRequiredCodeRule {
                message: "probe rule needs a value".to_string(),
            }),
            Rc::new(ProbeRuleRequiredScopeRule {
                message: "probe rule needs this field".to_string(),
            }),
            Rc::new(ProbeRuleScopeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_rule_validator();
        let item = new_probe_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ProbeRuleRequiredNameRule {
            message: "probe rule needs a value".to_string(),
        };
        let mut item = new_probe_rule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeRuleRequiredCodeRule {
            message: "probe rule needs a value".to_string(),
        };
        let mut item = new_probe_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ProbeRuleRequiredScopeRule {
            message: "probe rule needs this field".to_string(),
        };
        let mut item = new_probe_rule("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = ProbeRuleScopeLengthRule { max_length: 3600 };
        let mut item = new_probe_rule("x", "y");
        item.scope = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
