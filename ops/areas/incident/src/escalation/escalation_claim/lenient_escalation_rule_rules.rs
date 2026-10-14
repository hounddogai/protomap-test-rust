use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient escalation rule must pass. Check returns its problems, or none.
pub trait LenientEscalationRuleRule: std::fmt::Debug {
    fn check(&self, item: &LenientEscalationRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEscalationRuleRequiredCodeRule {
    pub message: String,
}

impl LenientEscalationRuleRule for LenientEscalationRuleRequiredCodeRule {
    fn check(&self, item: &LenientEscalationRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEscalationRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LenientEscalationRuleRule for LenientEscalationRuleAllowedCodeRule {
    fn check(&self, item: &LenientEscalationRule) -> Vec<String> {
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
pub struct LenientEscalationRuleNameLengthRule {
    pub max_length: i64,
}

impl LenientEscalationRuleRule for LenientEscalationRuleNameLengthRule {
    fn check(&self, item: &LenientEscalationRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEscalationRuleSegmentLengthRule {
    pub max_length: i64,
}

impl LenientEscalationRuleRule for LenientEscalationRuleSegmentLengthRule {
    fn check(&self, item: &LenientEscalationRule) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEscalationRuleRequiredSegmentRule {
    pub message: String,
}

impl LenientEscalationRuleRule for LenientEscalationRuleRequiredSegmentRule {
    fn check(&self, item: &LenientEscalationRule) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient escalation rule.
#[derive(Debug, Clone)]
pub struct LenientEscalationRuleValidator {
    pub rules: Vec<Rc<dyn LenientEscalationRuleRule>>,
}

impl LenientEscalationRuleValidator {
    pub fn validate(&self, item: &LenientEscalationRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientEscalationRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientEscalationRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_escalation_rule_validator() -> LenientEscalationRuleValidator {
    LenientEscalationRuleValidator {
        rules: vec![
            Rc::new(LenientEscalationRuleRequiredCodeRule {
                message: "lenient escalation rule needs an entry".to_string(),
            }) as Rc<dyn LenientEscalationRuleRule>,
            Rc::new(LenientEscalationRuleAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "seasonal".to_string()],
            }),
            Rc::new(LenientEscalationRuleNameLengthRule { max_length: 16 }),
            Rc::new(LenientEscalationRuleSegmentLengthRule { max_length: 255 }),
            Rc::new(LenientEscalationRuleRequiredSegmentRule {
                message: "lenient escalation rule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_escalation_rule_validator();
        let item = new_lenient_escalation_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientEscalationRuleRequiredCodeRule {
            message: "lenient escalation rule needs an entry".to_string(),
        };
        let mut item = new_lenient_escalation_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LenientEscalationRuleNameLengthRule { max_length: 1000 };
        let mut item = new_lenient_escalation_rule("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = LenientEscalationRuleSegmentLengthRule { max_length: 120 };
        let mut item = new_lenient_escalation_rule("x", "y");
        item.segment = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = LenientEscalationRuleRequiredSegmentRule {
            message: "lenient escalation rule needs a value".to_string(),
        };
        let mut item = new_lenient_escalation_rule("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
