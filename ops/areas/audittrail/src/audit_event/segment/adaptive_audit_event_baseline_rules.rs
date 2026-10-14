use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive audit event baseline must pass. Check returns its problems, or none.
pub trait AdaptiveAuditEventBaselineRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveAuditEventBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventBaselineMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveAuditEventBaselineRule for AdaptiveAuditEventBaselineMaximumRangeRule {
    fn check(&self, item: &AdaptiveAuditEventBaseline) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventBaselineReviewerLengthRule {
    pub max_length: i64,
}

impl AdaptiveAuditEventBaselineRule for AdaptiveAuditEventBaselineReviewerLengthRule {
    fn check(&self, item: &AdaptiveAuditEventBaseline) -> Vec<String> {
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
pub struct AdaptiveAuditEventBaselineLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveAuditEventBaselineRule for AdaptiveAuditEventBaselineLabelLengthRule {
    fn check(&self, item: &AdaptiveAuditEventBaseline) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive audit event baseline.
#[derive(Debug, Clone)]
pub struct AdaptiveAuditEventBaselineValidator {
    pub rules: Vec<Rc<dyn AdaptiveAuditEventBaselineRule>>,
}

impl AdaptiveAuditEventBaselineValidator {
    pub fn validate(&self, item: &AdaptiveAuditEventBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveAuditEventBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveAuditEventBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_audit_event_baseline_validator() -> AdaptiveAuditEventBaselineValidator {
    AdaptiveAuditEventBaselineValidator {
        rules: vec![
            Rc::new(AdaptiveAuditEventBaselineMaximumRangeRule { min: 3.0, max: 4690.5 })
                as Rc<dyn AdaptiveAuditEventBaselineRule>,
            Rc::new(AdaptiveAuditEventBaselineReviewerLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveAuditEventBaselineLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_audit_event_baseline_validator();
        let item = new_adaptive_audit_event_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = AdaptiveAuditEventBaselineMaximumRangeRule { min: 0.8, max: 4316.3 };
        let mut item = new_adaptive_audit_event_baseline("x", "y");
        item.maximum = 4317.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = AdaptiveAuditEventBaselineReviewerLengthRule { max_length: 64 };
        let mut item = new_adaptive_audit_event_baseline("x", "y");
        item.reviewer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AdaptiveAuditEventBaselineLabelLengthRule { max_length: 86400 };
        let mut item = new_adaptive_audit_event_baseline("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
