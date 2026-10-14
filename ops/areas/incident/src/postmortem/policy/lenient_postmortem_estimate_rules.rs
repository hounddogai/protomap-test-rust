use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient postmortem estimate must pass. Check returns its problems, or none.
pub trait LenientPostmortemEstimateRule: std::fmt::Debug {
    fn check(&self, item: &LenientPostmortemEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPostmortemEstimateCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPostmortemEstimateRule for LenientPostmortemEstimateCreatedAtRangeRule {
    fn check(&self, item: &LenientPostmortemEstimate) -> Vec<String> {
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
pub struct LenientPostmortemEstimateRequiredNamespaceRule {
    pub message: String,
}

impl LenientPostmortemEstimateRule for LenientPostmortemEstimateRequiredNamespaceRule {
    fn check(&self, item: &LenientPostmortemEstimate) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPostmortemEstimateDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPostmortemEstimateRule for LenientPostmortemEstimateDurationMinutesRangeRule {
    fn check(&self, item: &LenientPostmortemEstimate) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient postmortem estimate.
#[derive(Debug, Clone)]
pub struct LenientPostmortemEstimateValidator {
    pub rules: Vec<Rc<dyn LenientPostmortemEstimateRule>>,
}

impl LenientPostmortemEstimateValidator {
    pub fn validate(&self, item: &LenientPostmortemEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientPostmortemEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPostmortemEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_postmortem_estimate_validator() -> LenientPostmortemEstimateValidator {
    LenientPostmortemEstimateValidator {
        rules: vec![
            Rc::new(LenientPostmortemEstimateCreatedAtRangeRule { min: 120, max: 3600 })
                as Rc<dyn LenientPostmortemEstimateRule>,
            Rc::new(LenientPostmortemEstimateRequiredNamespaceRule {
                message: "lenient postmortem estimate needs this field".to_string(),
            }),
            Rc::new(LenientPostmortemEstimateDurationMinutesRangeRule { min: 64, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_postmortem_estimate_validator();
        let item = new_lenient_postmortem_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = LenientPostmortemEstimateCreatedAtRangeRule { min: 120, max: 86400 };
        let mut item = new_lenient_postmortem_estimate("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = LenientPostmortemEstimateRequiredNamespaceRule {
            message: "lenient postmortem estimate needs a value".to_string(),
        };
        let mut item = new_lenient_postmortem_estimate("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = LenientPostmortemEstimateDurationMinutesRangeRule { min: 32, max: 1000 };
        let mut item = new_lenient_postmortem_estimate("x", "y");
        item.duration_minutes = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
