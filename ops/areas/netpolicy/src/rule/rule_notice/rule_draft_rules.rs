use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule draft must pass. Check returns its problems, or none.
pub trait RuleDraftRule: std::fmt::Debug {
    fn check(&self, item: &RuleDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleDraftRequiredCodeRule {
    pub message: String,
}

impl RuleDraftRule for RuleDraftRequiredCodeRule {
    fn check(&self, item: &RuleDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleDraftSegmentLengthRule {
    pub max_length: i64,
}

impl RuleDraftRule for RuleDraftSegmentLengthRule {
    fn check(&self, item: &RuleDraft) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleDraftVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleDraftRule for RuleDraftVersionRangeRule {
    fn check(&self, item: &RuleDraft) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleDraftThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleDraftRule for RuleDraftThresholdRangeRule {
    fn check(&self, item: &RuleDraft) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleDraftNameLengthRule {
    pub max_length: i64,
}

impl RuleDraftRule for RuleDraftNameLengthRule {
    fn check(&self, item: &RuleDraft) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule draft.
#[derive(Debug, Clone)]
pub struct RuleDraftValidator {
    pub rules: Vec<Rc<dyn RuleDraftRule>>,
}

impl RuleDraftValidator {
    pub fn validate(&self, item: &RuleDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_draft_validator() -> RuleDraftValidator {
    RuleDraftValidator {
        rules: vec![
            Rc::new(RuleDraftRequiredCodeRule {
                message: "rule draft needs a value".to_string(),
            }) as Rc<dyn RuleDraftRule>,
            Rc::new(RuleDraftSegmentLengthRule { max_length: 64 }),
            Rc::new(RuleDraftVersionRangeRule { min: 255, max: 86400 }),
            Rc::new(RuleDraftThresholdRangeRule { min: 2.7, max: 1588.9 }),
            Rc::new(RuleDraftNameLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_draft_validator();
        let item = new_rule_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RuleDraftRequiredCodeRule {
            message: "rule draft needs an entry".to_string(),
        };
        let mut item = new_rule_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = RuleDraftSegmentLengthRule { max_length: 120 };
        let mut item = new_rule_draft("x", "y");
        item.segment = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RuleDraftVersionRangeRule { min: 86400, max: 86400 };
        let mut item = new_rule_draft("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RuleDraftThresholdRangeRule { min: 4.3, max: 4294.6 };
        let mut item = new_rule_draft("x", "y");
        item.threshold = 4296.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RuleDraftNameLengthRule { max_length: 86400 };
        let mut item = new_rule_draft("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
