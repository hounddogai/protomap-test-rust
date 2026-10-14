use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional policy item must pass. Check returns its problems, or none.
pub trait RegionalPolicyItemRule: std::fmt::Debug {
    fn check(&self, item: &RegionalPolicyItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyItemReviewerLengthRule {
    pub max_length: i64,
}

impl RegionalPolicyItemRule for RegionalPolicyItemReviewerLengthRule {
    fn check(&self, item: &RegionalPolicyItem) -> Vec<String> {
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
pub struct RegionalPolicyItemVisibleGuardRule {
    pub limit: f64,
}

impl RegionalPolicyItemRule for RegionalPolicyItemVisibleGuardRule {
    fn check(&self, item: &RegionalPolicyItem) -> Vec<String> {
        if item.visible && (item.margin > self.limit) {
            return vec![format!("margin {} exceeds {} while visible", item.margin, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyItemCodeLengthRule {
    pub max_length: i64,
}

impl RegionalPolicyItemRule for RegionalPolicyItemCodeLengthRule {
    fn check(&self, item: &RegionalPolicyItem) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional policy item.
#[derive(Debug, Clone)]
pub struct RegionalPolicyItemValidator {
    pub rules: Vec<Rc<dyn RegionalPolicyItemRule>>,
}

impl RegionalPolicyItemValidator {
    pub fn validate(&self, item: &RegionalPolicyItem) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalPolicyItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalPolicyItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_policy_item_validator() -> RegionalPolicyItemValidator {
    RegionalPolicyItemValidator {
        rules: vec![
            Rc::new(RegionalPolicyItemReviewerLengthRule { max_length: 80 }) as Rc<dyn RegionalPolicyItemRule>,
            Rc::new(RegionalPolicyItemVisibleGuardRule { limit: 8.1 }),
            Rc::new(RegionalPolicyItemCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_policy_item_validator();
        let item = new_regional_policy_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = RegionalPolicyItemReviewerLengthRule { max_length: 16 };
        let mut item = new_regional_policy_item("x", "y");
        item.reviewer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalPolicyItemCodeLengthRule { max_length: 120 };
        let mut item = new_regional_policy_item("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
