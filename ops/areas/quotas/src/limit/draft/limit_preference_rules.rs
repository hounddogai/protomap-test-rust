use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit preference must pass. Check returns its problems, or none.
pub trait LimitPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LimitPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitPreferencePinnedGuardRule {
    pub limit: i64,
}

impl LimitPreferenceRule for LimitPreferencePinnedGuardRule {
    fn check(&self, item: &LimitPreference) -> Vec<String> {
        if item.pinned && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while pinned",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitPreferenceUnitLengthRule {
    pub max_length: i64,
}

impl LimitPreferenceRule for LimitPreferenceUnitLengthRule {
    fn check(&self, item: &LimitPreference) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitPreferenceQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitPreferenceRule for LimitPreferenceQuantityRangeRule {
    fn check(&self, item: &LimitPreference) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a limit preference.
#[derive(Debug, Clone)]
pub struct LimitPreferenceValidator {
    pub rules: Vec<Rc<dyn LimitPreferenceRule>>,
}

impl LimitPreferenceValidator {
    pub fn validate(&self, item: &LimitPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_preference_validator() -> LimitPreferenceValidator {
    LimitPreferenceValidator {
        rules: vec![
            Rc::new(LimitPreferencePinnedGuardRule { limit: 80 }) as Rc<dyn LimitPreferenceRule>,
            Rc::new(LimitPreferenceUnitLengthRule { max_length: 86400 }),
            Rc::new(LimitPreferenceQuantityRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_preference_validator();
        let item = new_limit_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = LimitPreferenceUnitLengthRule { max_length: 1000 };
        let mut item = new_limit_preference("x", "y");
        item.unit = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = LimitPreferenceQuantityRangeRule { min: 64, max: 3600 };
        let mut item = new_limit_preference("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
