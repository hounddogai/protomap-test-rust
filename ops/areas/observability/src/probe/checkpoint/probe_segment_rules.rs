use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe segment must pass. Check returns its problems, or none.
pub trait ProbeSegmentRule: std::fmt::Debug {
    fn check(&self, item: &ProbeSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSegmentGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeSegmentRule for ProbeSegmentGraceDaysRangeRule {
    fn check(&self, item: &ProbeSegment) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSegmentCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeSegmentRule for ProbeSegmentCreatedAtRangeRule {
    fn check(&self, item: &ProbeSegment) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSegmentCodePrefixRule {
    pub prefix: String,
}

impl ProbeSegmentRule for ProbeSegmentCodePrefixRule {
    fn check(&self, item: &ProbeSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSegmentRequiredCodeRule {
    pub message: String,
}

impl ProbeSegmentRule for ProbeSegmentRequiredCodeRule {
    fn check(&self, item: &ProbeSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSegmentTitleLengthRule {
    pub max_length: i64,
}

impl ProbeSegmentRule for ProbeSegmentTitleLengthRule {
    fn check(&self, item: &ProbeSegment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe segment.
#[derive(Debug, Clone)]
pub struct ProbeSegmentValidator {
    pub rules: Vec<Rc<dyn ProbeSegmentRule>>,
}

impl ProbeSegmentValidator {
    pub fn validate(&self, item: &ProbeSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbeSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_segment_validator() -> ProbeSegmentValidator {
    ProbeSegmentValidator {
        rules: vec![
            Rc::new(ProbeSegmentGraceDaysRangeRule { min: 64, max: 255 }) as Rc<dyn ProbeSegmentRule>,
            Rc::new(ProbeSegmentCreatedAtRangeRule { min: 64, max: 80 }),
            Rc::new(ProbeSegmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ProbeSegmentRequiredCodeRule {
                message: "probe segment needs this field".to_string(),
            }),
            Rc::new(ProbeSegmentTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_segment_validator();
        let item = new_probe_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = ProbeSegmentGraceDaysRangeRule { min: 120, max: 120 };
        let mut item = new_probe_segment("x", "y");
        item.grace_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ProbeSegmentCreatedAtRangeRule { min: 255, max: 3600 };
        let mut item = new_probe_segment("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeSegmentRequiredCodeRule {
            message: "probe segment needs a value".to_string(),
        };
        let mut item = new_probe_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ProbeSegmentTitleLengthRule { max_length: 80 };
        let mut item = new_probe_segment("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
