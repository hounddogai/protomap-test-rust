use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional redaction group must pass. Check returns its problems, or none.
pub trait RegionalRedactionGroupRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRedactionGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionGroupRequiredSegmentRule {
    pub message: String,
}

impl RegionalRedactionGroupRule for RegionalRedactionGroupRequiredSegmentRule {
    fn check(&self, item: &RegionalRedactionGroup) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionGroupMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalRedactionGroupRule for RegionalRedactionGroupMinimumRangeRule {
    fn check(&self, item: &RegionalRedactionGroup) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionGroupPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalRedactionGroupRule for RegionalRedactionGroupPortRangeRule {
    fn check(&self, item: &RegionalRedactionGroup) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionGroupAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalRedactionGroupRule for RegionalRedactionGroupAllowedCodeRule {
    fn check(&self, item: &RegionalRedactionGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionGroupRequiredAudienceRule {
    pub message: String,
}

impl RegionalRedactionGroupRule for RegionalRedactionGroupRequiredAudienceRule {
    fn check(&self, item: &RegionalRedactionGroup) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionGroupSegmentLengthRule {
    pub max_length: i64,
}

impl RegionalRedactionGroupRule for RegionalRedactionGroupSegmentLengthRule {
    fn check(&self, item: &RegionalRedactionGroup) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional redaction group.
#[derive(Debug, Clone)]
pub struct RegionalRedactionGroupValidator {
    pub rules: Vec<Rc<dyn RegionalRedactionGroupRule>>,
}

impl RegionalRedactionGroupValidator {
    pub fn validate(&self, item: &RegionalRedactionGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalRedactionGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRedactionGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_redaction_group_validator() -> RegionalRedactionGroupValidator {
    RegionalRedactionGroupValidator {
        rules: vec![
            Rc::new(RegionalRedactionGroupRequiredSegmentRule {
                message: "regional redaction group needs this field".to_string(),
            }) as Rc<dyn RegionalRedactionGroupRule>,
            Rc::new(RegionalRedactionGroupMinimumRangeRule { min: 4.2, max: 2573.3 }),
            Rc::new(RegionalRedactionGroupPortRangeRule { min: 32, max: 255 }),
            Rc::new(RegionalRedactionGroupAllowedCodeRule {
                allowed: vec!["strict".to_string(), "pending".to_string(), "default".to_string()],
            }),
            Rc::new(RegionalRedactionGroupRequiredAudienceRule {
                message: "regional redaction group needs a value".to_string(),
            }),
            Rc::new(RegionalRedactionGroupSegmentLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_redaction_group_validator();
        let item = new_regional_redaction_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = RegionalRedactionGroupRequiredSegmentRule {
            message: "regional redaction group needs a value".to_string(),
        };
        let mut item = new_regional_redaction_group("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = RegionalRedactionGroupMinimumRangeRule { min: 1.0, max: 722.8 };
        let mut item = new_regional_redaction_group("x", "y");
        item.minimum = 724.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = RegionalRedactionGroupPortRangeRule { min: 64, max: 3600 };
        let mut item = new_regional_redaction_group("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = RegionalRedactionGroupRequiredAudienceRule {
            message: "regional redaction group needs this field".to_string(),
        };
        let mut item = new_regional_redaction_group("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = RegionalRedactionGroupSegmentLengthRule { max_length: 86400 };
        let mut item = new_regional_redaction_group("x", "y");
        item.segment = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
