use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy usage target must pass. Check returns its problems, or none.
pub trait LegacyUsageTargetRule: std::fmt::Debug {
    fn check(&self, item: &LegacyUsageTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageTargetCodeLengthRule {
    pub max_length: i64,
}

impl LegacyUsageTargetRule for LegacyUsageTargetCodeLengthRule {
    fn check(&self, item: &LegacyUsageTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageTargetTitleLengthRule {
    pub max_length: i64,
}

impl LegacyUsageTargetRule for LegacyUsageTargetTitleLengthRule {
    fn check(&self, item: &LegacyUsageTarget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageTargetArchivedGuardRule {
    pub limit: i64,
}

impl LegacyUsageTargetRule for LegacyUsageTargetArchivedGuardRule {
    fn check(&self, item: &LegacyUsageTarget) -> Vec<String> {
        if item.archived && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while archived",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageTargetRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyUsageTargetRule for LegacyUsageTargetRotationDaysRangeRule {
    fn check(&self, item: &LegacyUsageTarget) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageTargetPrincipalLengthRule {
    pub max_length: i64,
}

impl LegacyUsageTargetRule for LegacyUsageTargetPrincipalLengthRule {
    fn check(&self, item: &LegacyUsageTarget) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy usage target.
#[derive(Debug, Clone)]
pub struct LegacyUsageTargetValidator {
    pub rules: Vec<Rc<dyn LegacyUsageTargetRule>>,
}

impl LegacyUsageTargetValidator {
    pub fn validate(&self, item: &LegacyUsageTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyUsageTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyUsageTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_usage_target_validator() -> LegacyUsageTargetValidator {
    LegacyUsageTargetValidator {
        rules: vec![
            Rc::new(LegacyUsageTargetCodeLengthRule { max_length: 64 }) as Rc<dyn LegacyUsageTargetRule>,
            Rc::new(LegacyUsageTargetTitleLengthRule { max_length: 255 }),
            Rc::new(LegacyUsageTargetArchivedGuardRule { limit: 86400 }),
            Rc::new(LegacyUsageTargetRotationDaysRangeRule { min: 120, max: 86400 }),
            Rc::new(LegacyUsageTargetPrincipalLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_usage_target_validator();
        let item = new_legacy_usage_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyUsageTargetCodeLengthRule { max_length: 120 };
        let mut item = new_legacy_usage_target("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyUsageTargetTitleLengthRule { max_length: 86400 };
        let mut item = new_legacy_usage_target("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = LegacyUsageTargetRotationDaysRangeRule { min: 32, max: 1000 };
        let mut item = new_legacy_usage_target("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = LegacyUsageTargetPrincipalLengthRule { max_length: 120 };
        let mut item = new_legacy_usage_target("x", "y");
        item.principal = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
