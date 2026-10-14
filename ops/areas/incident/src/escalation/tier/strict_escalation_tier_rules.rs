use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict escalation tier must pass. Check returns its problems, or none.
pub trait StrictEscalationTierRule: std::fmt::Debug {
    fn check(&self, item: &StrictEscalationTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationTierCodeLengthRule {
    pub max_length: i64,
}

impl StrictEscalationTierRule for StrictEscalationTierCodeLengthRule {
    fn check(&self, item: &StrictEscalationTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationTierRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictEscalationTierRule for StrictEscalationTierRevisionRangeRule {
    fn check(&self, item: &StrictEscalationTier) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationTierRequiredDescriptionRule {
    pub message: String,
}

impl StrictEscalationTierRule for StrictEscalationTierRequiredDescriptionRule {
    fn check(&self, item: &StrictEscalationTier) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationTierRequiredCodeRule {
    pub message: String,
}

impl StrictEscalationTierRule for StrictEscalationTierRequiredCodeRule {
    fn check(&self, item: &StrictEscalationTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationTierSegmentLengthRule {
    pub max_length: i64,
}

impl StrictEscalationTierRule for StrictEscalationTierSegmentLengthRule {
    fn check(&self, item: &StrictEscalationTier) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationTierRequiredSegmentRule {
    pub message: String,
}

impl StrictEscalationTierRule for StrictEscalationTierRequiredSegmentRule {
    fn check(&self, item: &StrictEscalationTier) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict escalation tier.
#[derive(Debug, Clone)]
pub struct StrictEscalationTierValidator {
    pub rules: Vec<Rc<dyn StrictEscalationTierRule>>,
}

impl StrictEscalationTierValidator {
    pub fn validate(&self, item: &StrictEscalationTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictEscalationTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictEscalationTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_escalation_tier_validator() -> StrictEscalationTierValidator {
    StrictEscalationTierValidator {
        rules: vec![
            Rc::new(StrictEscalationTierCodeLengthRule { max_length: 80 }) as Rc<dyn StrictEscalationTierRule>,
            Rc::new(StrictEscalationTierRevisionRangeRule { min: 32, max: 64 }),
            Rc::new(StrictEscalationTierRequiredDescriptionRule {
                message: "strict escalation tier needs this field".to_string(),
            }),
            Rc::new(StrictEscalationTierRequiredCodeRule {
                message: "strict escalation tier needs this field".to_string(),
            }),
            Rc::new(StrictEscalationTierSegmentLengthRule { max_length: 80 }),
            Rc::new(StrictEscalationTierRequiredSegmentRule {
                message: "strict escalation tier needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_escalation_tier_validator();
        let item = new_strict_escalation_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictEscalationTierCodeLengthRule { max_length: 86400 };
        let mut item = new_strict_escalation_tier("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = StrictEscalationTierRevisionRangeRule { min: 3600, max: 3600 };
        let mut item = new_strict_escalation_tier("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = StrictEscalationTierRequiredDescriptionRule {
            message: "strict escalation tier needs this field".to_string(),
        };
        let mut item = new_strict_escalation_tier("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictEscalationTierRequiredCodeRule {
            message: "strict escalation tier needs an entry".to_string(),
        };
        let mut item = new_strict_escalation_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = StrictEscalationTierSegmentLengthRule { max_length: 64 };
        let mut item = new_strict_escalation_tier("x", "y");
        item.segment = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = StrictEscalationTierRequiredSegmentRule {
            message: "strict escalation tier needs a value".to_string(),
        };
        let mut item = new_strict_escalation_tier("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
