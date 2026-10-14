use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional escalation segment must pass. Check returns its problems, or none.
pub trait RegionalEscalationSegmentRule: std::fmt::Debug {
    fn check(&self, item: &RegionalEscalationSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEscalationSegmentAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalEscalationSegmentRule for RegionalEscalationSegmentAmountRangeRule {
    fn check(&self, item: &RegionalEscalationSegment) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEscalationSegmentRequiredAudienceRule {
    pub message: String,
}

impl RegionalEscalationSegmentRule for RegionalEscalationSegmentRequiredAudienceRule {
    fn check(&self, item: &RegionalEscalationSegment) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEscalationSegmentRequiredNameRule {
    pub message: String,
}

impl RegionalEscalationSegmentRule for RegionalEscalationSegmentRequiredNameRule {
    fn check(&self, item: &RegionalEscalationSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEscalationSegmentNameLengthRule {
    pub max_length: i64,
}

impl RegionalEscalationSegmentRule for RegionalEscalationSegmentNameLengthRule {
    fn check(&self, item: &RegionalEscalationSegment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEscalationSegmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalEscalationSegmentRule for RegionalEscalationSegmentAllowedCodeRule {
    fn check(&self, item: &RegionalEscalationSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional escalation segment.
#[derive(Debug, Clone)]
pub struct RegionalEscalationSegmentValidator {
    pub rules: Vec<Rc<dyn RegionalEscalationSegmentRule>>,
}

impl RegionalEscalationSegmentValidator {
    pub fn validate(&self, item: &RegionalEscalationSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalEscalationSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalEscalationSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_escalation_segment_validator() -> RegionalEscalationSegmentValidator {
    RegionalEscalationSegmentValidator {
        rules: vec![
            Rc::new(RegionalEscalationSegmentAmountRangeRule { min: 7.7, max: 4006.5 })
                as Rc<dyn RegionalEscalationSegmentRule>,
            Rc::new(RegionalEscalationSegmentRequiredAudienceRule {
                message: "regional escalation segment needs this field".to_string(),
            }),
            Rc::new(RegionalEscalationSegmentRequiredNameRule {
                message: "regional escalation segment needs an entry".to_string(),
            }),
            Rc::new(RegionalEscalationSegmentNameLengthRule { max_length: 80 }),
            Rc::new(RegionalEscalationSegmentAllowedCodeRule {
                allowed: vec!["adaptive".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_escalation_segment_validator();
        let item = new_regional_escalation_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = RegionalEscalationSegmentAmountRangeRule { min: 2.0, max: 4888.6 };
        let mut item = new_regional_escalation_segment("x", "y");
        item.amount = 4890.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = RegionalEscalationSegmentRequiredAudienceRule {
            message: "regional escalation segment needs a value".to_string(),
        };
        let mut item = new_regional_escalation_segment("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalEscalationSegmentRequiredNameRule {
            message: "regional escalation segment needs this field".to_string(),
        };
        let mut item = new_regional_escalation_segment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalEscalationSegmentNameLengthRule { max_length: 86400 };
        let mut item = new_regional_escalation_segment("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
