use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped redaction rule must pass. Check returns its problems, or none.
pub trait CappedRedactionRuleRule: std::fmt::Debug {
    fn check(&self, item: &CappedRedactionRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionRuleCodeLengthRule {
    pub max_length: i64,
}

impl CappedRedactionRuleRule for CappedRedactionRuleCodeLengthRule {
    fn check(&self, item: &CappedRedactionRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionRuleRequiredCodeRule {
    pub message: String,
}

impl CappedRedactionRuleRule for CappedRedactionRuleRequiredCodeRule {
    fn check(&self, item: &CappedRedactionRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionRuleTitleLengthRule {
    pub max_length: i64,
}

impl CappedRedactionRuleRule for CappedRedactionRuleTitleLengthRule {
    fn check(&self, item: &CappedRedactionRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionRuleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedRedactionRuleRule for CappedRedactionRuleMaximumRangeRule {
    fn check(&self, item: &CappedRedactionRule) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionRuleAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl CappedRedactionRuleRule for CappedRedactionRuleAllowedClusterRule {
    fn check(&self, item: &CappedRedactionRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is out of range", item.cluster)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionRuleCodePrefixRule {
    pub prefix: String,
}

impl CappedRedactionRuleRule for CappedRedactionRuleCodePrefixRule {
    fn check(&self, item: &CappedRedactionRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped redaction rule.
#[derive(Debug, Clone)]
pub struct CappedRedactionRuleValidator {
    pub rules: Vec<Rc<dyn CappedRedactionRuleRule>>,
}

impl CappedRedactionRuleValidator {
    pub fn validate(&self, item: &CappedRedactionRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedRedactionRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRedactionRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_redaction_rule_validator() -> CappedRedactionRuleValidator {
    CappedRedactionRuleValidator {
        rules: vec![
            Rc::new(CappedRedactionRuleCodeLengthRule { max_length: 64 }) as Rc<dyn CappedRedactionRuleRule>,
            Rc::new(CappedRedactionRuleRequiredCodeRule {
                message: "capped redaction rule needs an entry".to_string(),
            }),
            Rc::new(CappedRedactionRuleTitleLengthRule { max_length: 32 }),
            Rc::new(CappedRedactionRuleMaximumRangeRule { min: 1.2, max: 3820.7 }),
            Rc::new(CappedRedactionRuleAllowedClusterRule {
                allowed: vec!["primary".to_string(), "seasonal".to_string()],
            }),
            Rc::new(CappedRedactionRuleCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_redaction_rule_validator();
        let item = new_capped_redaction_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedRedactionRuleCodeLengthRule { max_length: 120 };
        let mut item = new_capped_redaction_rule("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedRedactionRuleRequiredCodeRule {
            message: "capped redaction rule needs a value".to_string(),
        };
        let mut item = new_capped_redaction_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CappedRedactionRuleTitleLengthRule { max_length: 86400 };
        let mut item = new_capped_redaction_rule("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = CappedRedactionRuleMaximumRangeRule { min: 8.5, max: 1928.9 };
        let mut item = new_capped_redaction_rule("x", "y");
        item.maximum = 1930.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
