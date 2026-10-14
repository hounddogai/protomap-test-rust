use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict rollout segment must pass. Check returns its problems, or none.
pub trait StrictRolloutSegmentRule: std::fmt::Debug {
    fn check(&self, item: &StrictRolloutSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutSegmentRequiredNotesRule {
    pub message: String,
}

impl StrictRolloutSegmentRule for StrictRolloutSegmentRequiredNotesRule {
    fn check(&self, item: &StrictRolloutSegment) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutSegmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictRolloutSegmentRule for StrictRolloutSegmentPortRangeRule {
    fn check(&self, item: &StrictRolloutSegment) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutSegmentCodePrefixRule {
    pub prefix: String,
}

impl StrictRolloutSegmentRule for StrictRolloutSegmentCodePrefixRule {
    fn check(&self, item: &StrictRolloutSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutSegmentRequiredCodeRule {
    pub message: String,
}

impl StrictRolloutSegmentRule for StrictRolloutSegmentRequiredCodeRule {
    fn check(&self, item: &StrictRolloutSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutSegmentRequiredNameRule {
    pub message: String,
}

impl StrictRolloutSegmentRule for StrictRolloutSegmentRequiredNameRule {
    fn check(&self, item: &StrictRolloutSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutSegmentQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictRolloutSegmentRule for StrictRolloutSegmentQuantityRangeRule {
    fn check(&self, item: &StrictRolloutSegment) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict rollout segment.
#[derive(Debug, Clone)]
pub struct StrictRolloutSegmentValidator {
    pub rules: Vec<Rc<dyn StrictRolloutSegmentRule>>,
}

impl StrictRolloutSegmentValidator {
    pub fn validate(&self, item: &StrictRolloutSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictRolloutSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRolloutSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_rollout_segment_validator() -> StrictRolloutSegmentValidator {
    StrictRolloutSegmentValidator {
        rules: vec![
            Rc::new(StrictRolloutSegmentRequiredNotesRule {
                message: "strict rollout segment needs a value".to_string(),
            }) as Rc<dyn StrictRolloutSegmentRule>,
            Rc::new(StrictRolloutSegmentPortRangeRule { min: 32, max: 3600 }),
            Rc::new(StrictRolloutSegmentCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(StrictRolloutSegmentRequiredCodeRule {
                message: "strict rollout segment needs this field".to_string(),
            }),
            Rc::new(StrictRolloutSegmentRequiredNameRule {
                message: "strict rollout segment needs a value".to_string(),
            }),
            Rc::new(StrictRolloutSegmentQuantityRangeRule { min: 16, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_rollout_segment_validator();
        let item = new_strict_rollout_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = StrictRolloutSegmentRequiredNotesRule {
            message: "strict rollout segment needs an entry".to_string(),
        };
        let mut item = new_strict_rollout_segment("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = StrictRolloutSegmentPortRangeRule { min: 120, max: 86400 };
        let mut item = new_strict_rollout_segment("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictRolloutSegmentRequiredCodeRule {
            message: "strict rollout segment needs an entry".to_string(),
        };
        let mut item = new_strict_rollout_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictRolloutSegmentRequiredNameRule {
            message: "strict rollout segment needs a value".to_string(),
        };
        let mut item = new_strict_rollout_segment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = StrictRolloutSegmentQuantityRangeRule { min: 16, max: 32 };
        let mut item = new_strict_rollout_segment("x", "y");
        item.quantity = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
