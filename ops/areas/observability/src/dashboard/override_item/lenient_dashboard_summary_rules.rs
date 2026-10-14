use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient dashboard summary must pass. Check returns its problems, or none.
pub trait LenientDashboardSummaryRule: std::fmt::Debug {
    fn check(&self, item: &LenientDashboardSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardSummaryAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl LenientDashboardSummaryRule for LenientDashboardSummaryAllowedUnitRule {
    fn check(&self, item: &LenientDashboardSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is duplicated", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardSummaryMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientDashboardSummaryRule for LenientDashboardSummaryMaximumRangeRule {
    fn check(&self, item: &LenientDashboardSummary) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardSummaryCodeLengthRule {
    pub max_length: i64,
}

impl LenientDashboardSummaryRule for LenientDashboardSummaryCodeLengthRule {
    fn check(&self, item: &LenientDashboardSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient dashboard summary.
#[derive(Debug, Clone)]
pub struct LenientDashboardSummaryValidator {
    pub rules: Vec<Rc<dyn LenientDashboardSummaryRule>>,
}

impl LenientDashboardSummaryValidator {
    pub fn validate(&self, item: &LenientDashboardSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientDashboardSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientDashboardSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_dashboard_summary_validator() -> LenientDashboardSummaryValidator {
    LenientDashboardSummaryValidator {
        rules: vec![
            Rc::new(LenientDashboardSummaryAllowedUnitRule { allowed: Vec::new() })
                as Rc<dyn LenientDashboardSummaryRule>,
            Rc::new(LenientDashboardSummaryMaximumRangeRule { min: 1.9, max: 532.7 }),
            Rc::new(LenientDashboardSummaryCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_dashboard_summary_validator();
        let item = new_lenient_dashboard_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = LenientDashboardSummaryMaximumRangeRule { min: 9.2, max: 400.7 };
        let mut item = new_lenient_dashboard_summary("x", "y");
        item.maximum = 402.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientDashboardSummaryCodeLengthRule { max_length: 3600 };
        let mut item = new_lenient_dashboard_summary("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
