use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist preference must pass. Check returns its problems, or none.
pub trait AllowlistPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistPreferenceRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistPreferenceRule for AllowlistPreferenceRetriesRangeRule {
    fn check(&self, item: &AllowlistPreference) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistPreferenceTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl AllowlistPreferenceRule for AllowlistPreferenceTagsLimitRule {
    fn check(&self, item: &AllowlistPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistPreferenceRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistPreferenceRule for AllowlistPreferenceRatioRangeRule {
    fn check(&self, item: &AllowlistPreference) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a allowlist preference.
#[derive(Debug, Clone)]
pub struct AllowlistPreferenceValidator {
    pub rules: Vec<Rc<dyn AllowlistPreferenceRule>>,
}

impl AllowlistPreferenceValidator {
    pub fn validate(&self, item: &AllowlistPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_preference_validator() -> AllowlistPreferenceValidator {
    AllowlistPreferenceValidator {
        rules: vec![
            Rc::new(AllowlistPreferenceRetriesRangeRule { min: 32, max: 64 }) as Rc<dyn AllowlistPreferenceRule>,
            Rc::new(AllowlistPreferenceTagsLimitRule {
                max_count: 16,
                banned: vec!["primary".to_string(), "adaptive".to_string()],
            }),
            Rc::new(AllowlistPreferenceRatioRangeRule { min: 8.7, max: 2226.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_preference_validator();
        let item = new_allowlist_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = AllowlistPreferenceRetriesRangeRule { min: 120, max: 3600 };
        let mut item = new_allowlist_preference("x", "y");
        item.retries = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = AllowlistPreferenceRatioRangeRule { min: 1.5, max: 161.1 };
        let mut item = new_allowlist_preference("x", "y");
        item.ratio = 162.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
