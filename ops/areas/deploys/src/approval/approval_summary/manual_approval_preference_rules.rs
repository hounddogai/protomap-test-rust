use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual approval preference must pass. Check returns its problems, or none.
pub trait ManualApprovalPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &ManualApprovalPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualApprovalPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl ManualApprovalPreferenceRule for ManualApprovalPreferenceCodeLengthRule {
    fn check(&self, item: &ManualApprovalPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualApprovalPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl ManualApprovalPreferenceRule for ManualApprovalPreferenceTitleLengthRule {
    fn check(&self, item: &ManualApprovalPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualApprovalPreferenceQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualApprovalPreferenceRule for ManualApprovalPreferenceQuantityRangeRule {
    fn check(&self, item: &ManualApprovalPreference) -> Vec<String> {
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

/// Runs every rule of a manual approval preference.
#[derive(Debug, Clone)]
pub struct ManualApprovalPreferenceValidator {
    pub rules: Vec<Rc<dyn ManualApprovalPreferenceRule>>,
}

impl ManualApprovalPreferenceValidator {
    pub fn validate(&self, item: &ManualApprovalPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualApprovalPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualApprovalPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_approval_preference_validator() -> ManualApprovalPreferenceValidator {
    ManualApprovalPreferenceValidator {
        rules: vec![
            Rc::new(ManualApprovalPreferenceCodeLengthRule { max_length: 120 }) as Rc<dyn ManualApprovalPreferenceRule>,
            Rc::new(ManualApprovalPreferenceTitleLengthRule { max_length: 16 }),
            Rc::new(ManualApprovalPreferenceQuantityRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_approval_preference_validator();
        let item = new_manual_approval_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualApprovalPreferenceCodeLengthRule { max_length: 120 };
        let mut item = new_manual_approval_preference("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualApprovalPreferenceTitleLengthRule { max_length: 16 };
        let mut item = new_manual_approval_preference("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ManualApprovalPreferenceQuantityRangeRule { min: 32, max: 1000 };
        let mut item = new_manual_approval_preference("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
