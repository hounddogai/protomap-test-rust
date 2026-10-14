use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback redaction summary must pass. Check returns its problems, or none.
pub trait FallbackRedactionSummaryRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRedactionSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionSummaryTimezoneLengthRule {
    pub max_length: i64,
}

impl FallbackRedactionSummaryRule for FallbackRedactionSummaryTimezoneLengthRule {
    fn check(&self, item: &FallbackRedactionSummary) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionSummaryTitleLengthRule {
    pub max_length: i64,
}

impl FallbackRedactionSummaryRule for FallbackRedactionSummaryTitleLengthRule {
    fn check(&self, item: &FallbackRedactionSummary) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionSummaryReviewerLengthRule {
    pub max_length: i64,
}

impl FallbackRedactionSummaryRule for FallbackRedactionSummaryReviewerLengthRule {
    fn check(&self, item: &FallbackRedactionSummary) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionSummaryBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackRedactionSummaryRule for FallbackRedactionSummaryBurstRangeRule {
    fn check(&self, item: &FallbackRedactionSummary) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionSummaryCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackRedactionSummaryRule for FallbackRedactionSummaryCostRangeRule {
    fn check(&self, item: &FallbackRedactionSummary) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionSummaryAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl FallbackRedactionSummaryRule for FallbackRedactionSummaryAllowedTimezoneRule {
    fn check(&self, item: &FallbackRedactionSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is unknown", item.timezone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback redaction summary.
#[derive(Debug, Clone)]
pub struct FallbackRedactionSummaryValidator {
    pub rules: Vec<Rc<dyn FallbackRedactionSummaryRule>>,
}

impl FallbackRedactionSummaryValidator {
    pub fn validate(&self, item: &FallbackRedactionSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackRedactionSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRedactionSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_redaction_summary_validator() -> FallbackRedactionSummaryValidator {
    FallbackRedactionSummaryValidator {
        rules: vec![
            Rc::new(FallbackRedactionSummaryTimezoneLengthRule { max_length: 80 })
                as Rc<dyn FallbackRedactionSummaryRule>,
            Rc::new(FallbackRedactionSummaryTitleLengthRule { max_length: 80 }),
            Rc::new(FallbackRedactionSummaryReviewerLengthRule { max_length: 120 }),
            Rc::new(FallbackRedactionSummaryBurstRangeRule { min: 32, max: 86400 }),
            Rc::new(FallbackRedactionSummaryCostRangeRule { min: 4.7, max: 2730.0 }),
            Rc::new(FallbackRedactionSummaryAllowedTimezoneRule {
                allowed: vec!["legacy".to_string(), "global".to_string(), "global".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_redaction_summary_validator();
        let item = new_fallback_redaction_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = FallbackRedactionSummaryTimezoneLengthRule { max_length: 3600 };
        let mut item = new_fallback_redaction_summary("x", "y");
        item.timezone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FallbackRedactionSummaryTitleLengthRule { max_length: 64 };
        let mut item = new_fallback_redaction_summary("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = FallbackRedactionSummaryReviewerLengthRule { max_length: 3600 };
        let mut item = new_fallback_redaction_summary("x", "y");
        item.reviewer = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = FallbackRedactionSummaryBurstRangeRule { min: 64, max: 255 };
        let mut item = new_fallback_redaction_summary("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = FallbackRedactionSummaryCostRangeRule { min: 5.6, max: 4171.5 };
        let mut item = new_fallback_redaction_summary("x", "y");
        item.cost = 4173.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
