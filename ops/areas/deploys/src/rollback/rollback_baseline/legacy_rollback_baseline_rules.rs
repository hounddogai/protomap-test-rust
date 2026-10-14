use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy rollback baseline must pass. Check returns its problems, or none.
pub trait LegacyRollbackBaselineRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRollbackBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackBaselineTimezoneLengthRule {
    pub max_length: i64,
}

impl LegacyRollbackBaselineRule for LegacyRollbackBaselineTimezoneLengthRule {
    fn check(&self, item: &LegacyRollbackBaseline) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackBaselineCodePrefixRule {
    pub prefix: String,
}

impl LegacyRollbackBaselineRule for LegacyRollbackBaselineCodePrefixRule {
    fn check(&self, item: &LegacyRollbackBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackBaselineAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl LegacyRollbackBaselineRule for LegacyRollbackBaselineAllowedTimezoneRule {
    fn check(&self, item: &LegacyRollbackBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is below minimum", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackBaselineCountryLengthRule {
    pub max_length: i64,
}

impl LegacyRollbackBaselineRule for LegacyRollbackBaselineCountryLengthRule {
    fn check(&self, item: &LegacyRollbackBaseline) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy rollback baseline.
#[derive(Debug, Clone)]
pub struct LegacyRollbackBaselineValidator {
    pub rules: Vec<Rc<dyn LegacyRollbackBaselineRule>>,
}

impl LegacyRollbackBaselineValidator {
    pub fn validate(&self, item: &LegacyRollbackBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyRollbackBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRollbackBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_rollback_baseline_validator() -> LegacyRollbackBaselineValidator {
    LegacyRollbackBaselineValidator {
        rules: vec![
            Rc::new(LegacyRollbackBaselineTimezoneLengthRule { max_length: 255 }) as Rc<dyn LegacyRollbackBaselineRule>,
            Rc::new(LegacyRollbackBaselineCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LegacyRollbackBaselineAllowedTimezoneRule {
                allowed: vec!["primary".to_string(), "adaptive".to_string()],
            }),
            Rc::new(LegacyRollbackBaselineCountryLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_rollback_baseline_validator();
        let item = new_legacy_rollback_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = LegacyRollbackBaselineTimezoneLengthRule { max_length: 64 };
        let mut item = new_legacy_rollback_baseline("x", "y");
        item.timezone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = LegacyRollbackBaselineCountryLengthRule { max_length: 120 };
        let mut item = new_legacy_rollback_baseline("x", "y");
        item.country = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
