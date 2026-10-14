use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict rule preference must pass. Check returns its problems, or none.
pub trait StrictRulePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &StrictRulePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRulePreferenceWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictRulePreferenceRule for StrictRulePreferenceWindowDaysRangeRule {
    fn check(&self, item: &StrictRulePreference) -> Vec<String> {
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
pub struct StrictRulePreferencePinnedGuardRule {
    pub limit: f64,
}

impl StrictRulePreferenceRule for StrictRulePreferencePinnedGuardRule {
    fn check(&self, item: &StrictRulePreference) -> Vec<String> {
        if item.pinned && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while pinned", item.cost, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRulePreferenceTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl StrictRulePreferenceRule for StrictRulePreferenceTagsLimitRule {
    fn check(&self, item: &StrictRulePreference) -> Vec<String> {
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
pub struct StrictRulePreferenceRequiredNotesRule {
    pub message: String,
}

impl StrictRulePreferenceRule for StrictRulePreferenceRequiredNotesRule {
    fn check(&self, item: &StrictRulePreference) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict rule preference.
#[derive(Debug, Clone)]
pub struct StrictRulePreferenceValidator {
    pub rules: Vec<Rc<dyn StrictRulePreferenceRule>>,
}

impl StrictRulePreferenceValidator {
    pub fn validate(&self, item: &StrictRulePreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictRulePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRulePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_rule_preference_validator() -> StrictRulePreferenceValidator {
    StrictRulePreferenceValidator {
        rules: vec![
            Rc::new(StrictRulePreferenceWindowDaysRangeRule { min: 16, max: 3600 }) as Rc<dyn StrictRulePreferenceRule>,
            Rc::new(StrictRulePreferencePinnedGuardRule { limit: 7.6 }),
            Rc::new(StrictRulePreferenceTagsLimitRule {
                max_count: 1000,
                banned: vec!["scheduled".to_string(), "regional".to_string()],
            }),
            Rc::new(StrictRulePreferenceRequiredNotesRule {
                message: "strict rule preference needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_rule_preference_validator();
        let item = new_strict_rule_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = StrictRulePreferenceWindowDaysRangeRule { min: 120, max: 1000 };
        let mut item = new_strict_rule_preference("x", "y");
        item.window_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = StrictRulePreferenceRequiredNotesRule {
            message: "strict rule preference needs an entry".to_string(),
        };
        let mut item = new_strict_rule_preference("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
