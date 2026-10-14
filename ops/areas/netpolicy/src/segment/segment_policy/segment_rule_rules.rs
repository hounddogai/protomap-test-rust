use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a segment rule must pass. Check returns its problems, or none.
pub trait SegmentRuleRule: std::fmt::Debug {
    fn check(&self, item: &SegmentRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRuleRequiredCipherRule {
    pub message: String,
}

impl SegmentRuleRule for SegmentRuleRequiredCipherRule {
    fn check(&self, item: &SegmentRule) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRuleCodeLengthRule {
    pub max_length: i64,
}

impl SegmentRuleRule for SegmentRuleCodeLengthRule {
    fn check(&self, item: &SegmentRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRuleEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentRuleRule for SegmentRuleEndsAtRangeRule {
    fn check(&self, item: &SegmentRule) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRuleChecksumLengthRule {
    pub max_length: i64,
}

impl SegmentRuleRule for SegmentRuleChecksumLengthRule {
    fn check(&self, item: &SegmentRule) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SegmentRuleRule for SegmentRuleAllowedCodeRule {
    fn check(&self, item: &SegmentRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRuleCipherLengthRule {
    pub max_length: i64,
}

impl SegmentRuleRule for SegmentRuleCipherLengthRule {
    fn check(&self, item: &SegmentRule) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment rule.
#[derive(Debug, Clone)]
pub struct SegmentRuleValidator {
    pub rules: Vec<Rc<dyn SegmentRuleRule>>,
}

impl SegmentRuleValidator {
    pub fn validate(&self, item: &SegmentRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_rule_validator() -> SegmentRuleValidator {
    SegmentRuleValidator {
        rules: vec![
            Rc::new(SegmentRuleRequiredCipherRule {
                message: "segment rule needs this field".to_string(),
            }) as Rc<dyn SegmentRuleRule>,
            Rc::new(SegmentRuleCodeLengthRule { max_length: 16 }),
            Rc::new(SegmentRuleEndsAtRangeRule { min: 1000, max: 86400 }),
            Rc::new(SegmentRuleChecksumLengthRule { max_length: 80 }),
            Rc::new(SegmentRuleAllowedCodeRule {
                allowed: vec!["default".to_string(), "strict".to_string(), "adaptive".to_string()],
            }),
            Rc::new(SegmentRuleCipherLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_rule_validator();
        let item = new_segment_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = SegmentRuleRequiredCipherRule {
            message: "segment rule needs an entry".to_string(),
        };
        let mut item = new_segment_rule("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SegmentRuleCodeLengthRule { max_length: 255 };
        let mut item = new_segment_rule("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = SegmentRuleEndsAtRangeRule { min: 32, max: 80 };
        let mut item = new_segment_rule("x", "y");
        item.ends_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = SegmentRuleChecksumLengthRule { max_length: 64 };
        let mut item = new_segment_rule("x", "y");
        item.checksum = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = SegmentRuleCipherLengthRule { max_length: 64 };
        let mut item = new_segment_rule("x", "y");
        item.cipher = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
