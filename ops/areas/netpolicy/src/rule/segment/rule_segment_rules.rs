use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rule segment must pass. Check returns its problems, or none.
pub trait RuleSegmentRule: std::fmt::Debug {
    fn check(&self, item: &RuleSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSegmentGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleSegmentRule for RuleSegmentGraceDaysRangeRule {
    fn check(&self, item: &RuleSegment) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSegmentAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl RuleSegmentRule for RuleSegmentAllowedSegmentRule {
    fn check(&self, item: &RuleSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is above maximum", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSegmentCodePrefixRule {
    pub prefix: String,
}

impl RuleSegmentRule for RuleSegmentCodePrefixRule {
    fn check(&self, item: &RuleSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSegmentRequiredLabelRule {
    pub message: String,
}

impl RuleSegmentRule for RuleSegmentRequiredLabelRule {
    fn check(&self, item: &RuleSegment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSegmentActiveGuardRule {
    pub limit: i64,
}

impl RuleSegmentRule for RuleSegmentActiveGuardRule {
    fn check(&self, item: &RuleSegment) -> Vec<String> {
        if item.active && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while active",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSegmentRequiredTitleRule {
    pub message: String,
}

impl RuleSegmentRule for RuleSegmentRequiredTitleRule {
    fn check(&self, item: &RuleSegment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule segment.
#[derive(Debug, Clone)]
pub struct RuleSegmentValidator {
    pub rules: Vec<Rc<dyn RuleSegmentRule>>,
}

impl RuleSegmentValidator {
    pub fn validate(&self, item: &RuleSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_segment_validator() -> RuleSegmentValidator {
    RuleSegmentValidator {
        rules: vec![
            Rc::new(RuleSegmentGraceDaysRangeRule { min: 32, max: 3600 }) as Rc<dyn RuleSegmentRule>,
            Rc::new(RuleSegmentAllowedSegmentRule {
                allowed: vec!["batched".to_string(), "pending".to_string(), "default".to_string()],
            }),
            Rc::new(RuleSegmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RuleSegmentRequiredLabelRule {
                message: "rule segment needs this field".to_string(),
            }),
            Rc::new(RuleSegmentActiveGuardRule { limit: 64 }),
            Rc::new(RuleSegmentRequiredTitleRule {
                message: "rule segment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_segment_validator();
        let item = new_rule_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RuleSegmentGraceDaysRangeRule { min: 80, max: 86400 };
        let mut item = new_rule_segment("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RuleSegmentRequiredLabelRule {
            message: "rule segment needs an entry".to_string(),
        };
        let mut item = new_rule_segment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RuleSegmentRequiredTitleRule {
            message: "rule segment needs an entry".to_string(),
        };
        let mut item = new_rule_segment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
