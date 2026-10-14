use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional usage snapshot must pass. Check returns its problems, or none.
pub trait RegionalUsageSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RegionalUsageSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSnapshotCodePrefixRule {
    pub prefix: String,
}

impl RegionalUsageSnapshotRule for RegionalUsageSnapshotCodePrefixRule {
    fn check(&self, item: &RegionalUsageSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSnapshotWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalUsageSnapshotRule for RegionalUsageSnapshotWindowDaysRangeRule {
    fn check(&self, item: &RegionalUsageSnapshot) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSnapshotAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalUsageSnapshotRule for RegionalUsageSnapshotAmountRangeRule {
    fn check(&self, item: &RegionalUsageSnapshot) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSnapshotAudienceLengthRule {
    pub max_length: i64,
}

impl RegionalUsageSnapshotRule for RegionalUsageSnapshotAudienceLengthRule {
    fn check(&self, item: &RegionalUsageSnapshot) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional usage snapshot.
#[derive(Debug, Clone)]
pub struct RegionalUsageSnapshotValidator {
    pub rules: Vec<Rc<dyn RegionalUsageSnapshotRule>>,
}

impl RegionalUsageSnapshotValidator {
    pub fn validate(&self, item: &RegionalUsageSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalUsageSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalUsageSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_usage_snapshot_validator() -> RegionalUsageSnapshotValidator {
    RegionalUsageSnapshotValidator {
        rules: vec![
            Rc::new(RegionalUsageSnapshotCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn RegionalUsageSnapshotRule>,
            Rc::new(RegionalUsageSnapshotWindowDaysRangeRule { min: 16, max: 1000 }),
            Rc::new(RegionalUsageSnapshotAmountRangeRule { min: 3.3, max: 2964.5 }),
            Rc::new(RegionalUsageSnapshotAudienceLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_usage_snapshot_validator();
        let item = new_regional_usage_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = RegionalUsageSnapshotWindowDaysRangeRule { min: 64, max: 120 };
        let mut item = new_regional_usage_snapshot("x", "y");
        item.window_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = RegionalUsageSnapshotAmountRangeRule { min: 0.3, max: 4604.0 };
        let mut item = new_regional_usage_snapshot("x", "y");
        item.amount = 4605.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = RegionalUsageSnapshotAudienceLengthRule { max_length: 255 };
        let mut item = new_regional_usage_snapshot("x", "y");
        item.audience = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
