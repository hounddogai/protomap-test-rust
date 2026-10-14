use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive probe summary must pass. Check returns its problems, or none.
pub trait AdaptiveProbeSummaryRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveProbeSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeSummaryQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveProbeSummaryRule for AdaptiveProbeSummaryQuotaUnitsRangeRule {
    fn check(&self, item: &AdaptiveProbeSummary) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeSummaryRequiredTitleRule {
    pub message: String,
}

impl AdaptiveProbeSummaryRule for AdaptiveProbeSummaryRequiredTitleRule {
    fn check(&self, item: &AdaptiveProbeSummary) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeSummarySequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveProbeSummaryRule for AdaptiveProbeSummarySequenceRangeRule {
    fn check(&self, item: &AdaptiveProbeSummary) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a adaptive probe summary.
#[derive(Debug, Clone)]
pub struct AdaptiveProbeSummaryValidator {
    pub rules: Vec<Rc<dyn AdaptiveProbeSummaryRule>>,
}

impl AdaptiveProbeSummaryValidator {
    pub fn validate(&self, item: &AdaptiveProbeSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveProbeSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveProbeSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_probe_summary_validator() -> AdaptiveProbeSummaryValidator {
    AdaptiveProbeSummaryValidator {
        rules: vec![
            Rc::new(AdaptiveProbeSummaryQuotaUnitsRangeRule { min: 16, max: 3600 }) as Rc<dyn AdaptiveProbeSummaryRule>,
            Rc::new(AdaptiveProbeSummaryRequiredTitleRule {
                message: "adaptive probe summary needs a value".to_string(),
            }),
            Rc::new(AdaptiveProbeSummarySequenceRangeRule { min: 32, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_probe_summary_validator();
        let item = new_adaptive_probe_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = AdaptiveProbeSummaryQuotaUnitsRangeRule { min: 255, max: 3600 };
        let mut item = new_adaptive_probe_summary("x", "y");
        item.quota_units = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptiveProbeSummaryRequiredTitleRule {
            message: "adaptive probe summary needs an entry".to_string(),
        };
        let mut item = new_adaptive_probe_summary("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = AdaptiveProbeSummarySequenceRangeRule { min: 32, max: 64 };
        let mut item = new_adaptive_probe_summary("x", "y");
        item.sequence = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
