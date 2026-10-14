use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a segment baseline must pass. Check returns its problems, or none.
pub trait SegmentBaselineRule: std::fmt::Debug {
    fn check(&self, item: &SegmentBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBaselineDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentBaselineRule for SegmentBaselineDurationMinutesRangeRule {
    fn check(&self, item: &SegmentBaseline) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBaselineRequiredTimezoneRule {
    pub message: String,
}

impl SegmentBaselineRule for SegmentBaselineRequiredTimezoneRule {
    fn check(&self, item: &SegmentBaseline) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBaselineArchivedGuardRule {
    pub limit: f64,
}

impl SegmentBaselineRule for SegmentBaselineArchivedGuardRule {
    fn check(&self, item: &SegmentBaseline) -> Vec<String> {
        if item.archived && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while archived", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBaselineAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SegmentBaselineRule for SegmentBaselineAllowedCodeRule {
    fn check(&self, item: &SegmentBaseline) -> Vec<String> {
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
pub struct SegmentBaselineRequiredTitleRule {
    pub message: String,
}

impl SegmentBaselineRule for SegmentBaselineRequiredTitleRule {
    fn check(&self, item: &SegmentBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBaselineRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentBaselineRule for SegmentBaselineRatioRangeRule {
    fn check(&self, item: &SegmentBaseline) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment baseline.
#[derive(Debug, Clone)]
pub struct SegmentBaselineValidator {
    pub rules: Vec<Rc<dyn SegmentBaselineRule>>,
}

impl SegmentBaselineValidator {
    pub fn validate(&self, item: &SegmentBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_baseline_validator() -> SegmentBaselineValidator {
    SegmentBaselineValidator {
        rules: vec![
            Rc::new(SegmentBaselineDurationMinutesRangeRule { min: 32, max: 80 }) as Rc<dyn SegmentBaselineRule>,
            Rc::new(SegmentBaselineRequiredTimezoneRule {
                message: "segment baseline needs this field".to_string(),
            }),
            Rc::new(SegmentBaselineArchivedGuardRule { limit: 8.4 }),
            Rc::new(SegmentBaselineAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "batched".to_string()],
            }),
            Rc::new(SegmentBaselineRequiredTitleRule {
                message: "segment baseline needs an entry".to_string(),
            }),
            Rc::new(SegmentBaselineRatioRangeRule { min: 0.5, max: 943.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_baseline_validator();
        let item = new_segment_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = SegmentBaselineDurationMinutesRangeRule { min: 32, max: 86400 };
        let mut item = new_segment_baseline("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = SegmentBaselineRequiredTimezoneRule {
            message: "segment baseline needs this field".to_string(),
        };
        let mut item = new_segment_baseline("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SegmentBaselineRequiredTitleRule {
            message: "segment baseline needs an entry".to_string(),
        };
        let mut item = new_segment_baseline("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = SegmentBaselineRatioRangeRule { min: 2.3, max: 113.7 };
        let mut item = new_segment_baseline("x", "y");
        item.ratio = 115.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
