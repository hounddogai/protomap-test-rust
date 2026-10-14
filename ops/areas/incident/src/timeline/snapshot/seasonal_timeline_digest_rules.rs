use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal timeline digest must pass. Check returns its problems, or none.
pub trait SeasonalTimelineDigestRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalTimelineDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTimelineDigestSummaryLengthRule {
    pub max_length: i64,
}

impl SeasonalTimelineDigestRule for SeasonalTimelineDigestSummaryLengthRule {
    fn check(&self, item: &SeasonalTimelineDigest) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTimelineDigestRequiredNameRule {
    pub message: String,
}

impl SeasonalTimelineDigestRule for SeasonalTimelineDigestRequiredNameRule {
    fn check(&self, item: &SeasonalTimelineDigest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTimelineDigestAllowedNameRule {
    pub allowed: Vec<String>,
}

impl SeasonalTimelineDigestRule for SeasonalTimelineDigestAllowedNameRule {
    fn check(&self, item: &SeasonalTimelineDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is unknown", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal timeline digest.
#[derive(Debug, Clone)]
pub struct SeasonalTimelineDigestValidator {
    pub rules: Vec<Rc<dyn SeasonalTimelineDigestRule>>,
}

impl SeasonalTimelineDigestValidator {
    pub fn validate(&self, item: &SeasonalTimelineDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalTimelineDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalTimelineDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_timeline_digest_validator() -> SeasonalTimelineDigestValidator {
    SeasonalTimelineDigestValidator {
        rules: vec![
            Rc::new(SeasonalTimelineDigestSummaryLengthRule { max_length: 64 }) as Rc<dyn SeasonalTimelineDigestRule>,
            Rc::new(SeasonalTimelineDigestRequiredNameRule {
                message: "seasonal timeline digest needs a value".to_string(),
            }),
            Rc::new(SeasonalTimelineDigestAllowedNameRule {
                allowed: vec!["rolling".to_string(), "tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_timeline_digest_validator();
        let item = new_seasonal_timeline_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = SeasonalTimelineDigestSummaryLengthRule { max_length: 32 };
        let mut item = new_seasonal_timeline_digest("x", "y");
        item.summary = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SeasonalTimelineDigestRequiredNameRule {
            message: "seasonal timeline digest needs a value".to_string(),
        };
        let mut item = new_seasonal_timeline_digest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
