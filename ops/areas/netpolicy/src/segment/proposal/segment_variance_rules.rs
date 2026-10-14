use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a segment variance must pass. Check returns its problems, or none.
pub trait SegmentVarianceRule: std::fmt::Debug {
    fn check(&self, item: &SegmentVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentVarianceRequiredHostRule {
    pub message: String,
}

impl SegmentVarianceRule for SegmentVarianceRequiredHostRule {
    fn check(&self, item: &SegmentVariance) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentVarianceMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentVarianceRule for SegmentVarianceMarginRangeRule {
    fn check(&self, item: &SegmentVariance) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentVarianceBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentVarianceRule for SegmentVarianceBurstRangeRule {
    fn check(&self, item: &SegmentVariance) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentVarianceCodeLengthRule {
    pub max_length: i64,
}

impl SegmentVarianceRule for SegmentVarianceCodeLengthRule {
    fn check(&self, item: &SegmentVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentVarianceAllowedHostRule {
    pub allowed: Vec<String>,
}

impl SegmentVarianceRule for SegmentVarianceAllowedHostRule {
    fn check(&self, item: &SegmentVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is inconsistent", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentVarianceTitleLengthRule {
    pub max_length: i64,
}

impl SegmentVarianceRule for SegmentVarianceTitleLengthRule {
    fn check(&self, item: &SegmentVariance) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment variance.
#[derive(Debug, Clone)]
pub struct SegmentVarianceValidator {
    pub rules: Vec<Rc<dyn SegmentVarianceRule>>,
}

impl SegmentVarianceValidator {
    pub fn validate(&self, item: &SegmentVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_variance_validator() -> SegmentVarianceValidator {
    SegmentVarianceValidator {
        rules: vec![
            Rc::new(SegmentVarianceRequiredHostRule {
                message: "segment variance needs an entry".to_string(),
            }) as Rc<dyn SegmentVarianceRule>,
            Rc::new(SegmentVarianceMarginRangeRule { min: 9.4, max: 3461.7 }),
            Rc::new(SegmentVarianceBurstRangeRule { min: 80, max: 1000 }),
            Rc::new(SegmentVarianceCodeLengthRule { max_length: 120 }),
            Rc::new(SegmentVarianceAllowedHostRule {
                allowed: vec!["capped".to_string(), "regional".to_string()],
            }),
            Rc::new(SegmentVarianceTitleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_variance_validator();
        let item = new_segment_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = SegmentVarianceRequiredHostRule {
            message: "segment variance needs a value".to_string(),
        };
        let mut item = new_segment_variance("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = SegmentVarianceMarginRangeRule { min: 2.5, max: 2318.6 };
        let mut item = new_segment_variance("x", "y");
        item.margin = 2320.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = SegmentVarianceBurstRangeRule { min: 32, max: 3600 };
        let mut item = new_segment_variance("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SegmentVarianceCodeLengthRule { max_length: 3600 };
        let mut item = new_segment_variance("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SegmentVarianceTitleLengthRule { max_length: 3600 };
        let mut item = new_segment_variance("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
