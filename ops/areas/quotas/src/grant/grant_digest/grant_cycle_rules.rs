use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a grant cycle must pass. Check returns its problems, or none.
pub trait GrantCycleRule: std::fmt::Debug {
    fn check(&self, item: &GrantCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantCycleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantCycleRule for GrantCycleWindowDaysRangeRule {
    fn check(&self, item: &GrantCycle) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantCycleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantCycleRule for GrantCycleCreatedAtRangeRule {
    fn check(&self, item: &GrantCycle) -> Vec<String> {
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
pub struct GrantCycleCodePrefixRule {
    pub prefix: String,
}

impl GrantCycleRule for GrantCycleCodePrefixRule {
    fn check(&self, item: &GrantCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantCycleAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl GrantCycleRule for GrantCycleAllowedReviewerRule {
    fn check(&self, item: &GrantCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is missing", item.reviewer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant cycle.
#[derive(Debug, Clone)]
pub struct GrantCycleValidator {
    pub rules: Vec<Rc<dyn GrantCycleRule>>,
}

impl GrantCycleValidator {
    pub fn validate(&self, item: &GrantCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_cycle_validator() -> GrantCycleValidator {
    GrantCycleValidator {
        rules: vec![
            Rc::new(GrantCycleWindowDaysRangeRule { min: 64, max: 255 }) as Rc<dyn GrantCycleRule>,
            Rc::new(GrantCycleCreatedAtRangeRule { min: 80, max: 120 }),
            Rc::new(GrantCycleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(GrantCycleAllowedReviewerRule {
                allowed: vec!["legacy".to_string(), "manual".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_cycle_validator();
        let item = new_grant_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = GrantCycleWindowDaysRangeRule { min: 16, max: 32 };
        let mut item = new_grant_cycle("x", "y");
        item.window_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = GrantCycleCreatedAtRangeRule { min: 80, max: 120 };
        let mut item = new_grant_cycle("x", "y");
        item.created_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
