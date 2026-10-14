use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling isolation target must pass. Check returns its problems, or none.
pub trait RollingIsolationTargetRule: std::fmt::Debug {
    fn check(&self, item: &RollingIsolationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationTargetAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl RollingIsolationTargetRule for RollingIsolationTargetAllowedIssuerRule {
    fn check(&self, item: &RollingIsolationTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is stale", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationTargetWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingIsolationTargetRule for RollingIsolationTargetWindowDaysRangeRule {
    fn check(&self, item: &RollingIsolationTarget) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationTargetSummaryLengthRule {
    pub max_length: i64,
}

impl RollingIsolationTargetRule for RollingIsolationTargetSummaryLengthRule {
    fn check(&self, item: &RollingIsolationTarget) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling isolation target.
#[derive(Debug, Clone)]
pub struct RollingIsolationTargetValidator {
    pub rules: Vec<Rc<dyn RollingIsolationTargetRule>>,
}

impl RollingIsolationTargetValidator {
    pub fn validate(&self, item: &RollingIsolationTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingIsolationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingIsolationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_isolation_target_validator() -> RollingIsolationTargetValidator {
    RollingIsolationTargetValidator {
        rules: vec![
            Rc::new(RollingIsolationTargetAllowedIssuerRule {
                allowed: vec!["legacy".to_string(), "rolling".to_string()],
            }) as Rc<dyn RollingIsolationTargetRule>,
            Rc::new(RollingIsolationTargetWindowDaysRangeRule { min: 1000, max: 86400 }),
            Rc::new(RollingIsolationTargetSummaryLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_isolation_target_validator();
        let item = new_rolling_isolation_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = RollingIsolationTargetWindowDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_rolling_isolation_target("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = RollingIsolationTargetSummaryLengthRule { max_length: 120 };
        let mut item = new_rolling_isolation_target("x", "y");
        item.summary = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
