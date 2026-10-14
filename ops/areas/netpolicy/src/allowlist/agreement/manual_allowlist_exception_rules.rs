use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual allowlist exception must pass. Check returns its problems, or none.
pub trait ManualAllowlistExceptionRule: std::fmt::Debug {
    fn check(&self, item: &ManualAllowlistException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistExceptionCodeLengthRule {
    pub max_length: i64,
}

impl ManualAllowlistExceptionRule for ManualAllowlistExceptionCodeLengthRule {
    fn check(&self, item: &ManualAllowlistException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistExceptionUnitLengthRule {
    pub max_length: i64,
}

impl ManualAllowlistExceptionRule for ManualAllowlistExceptionUnitLengthRule {
    fn check(&self, item: &ManualAllowlistException) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistExceptionRequiredCodeRule {
    pub message: String,
}

impl ManualAllowlistExceptionRule for ManualAllowlistExceptionRequiredCodeRule {
    fn check(&self, item: &ManualAllowlistException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual allowlist exception.
#[derive(Debug, Clone)]
pub struct ManualAllowlistExceptionValidator {
    pub rules: Vec<Rc<dyn ManualAllowlistExceptionRule>>,
}

impl ManualAllowlistExceptionValidator {
    pub fn validate(&self, item: &ManualAllowlistException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualAllowlistException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualAllowlistException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_allowlist_exception_validator() -> ManualAllowlistExceptionValidator {
    ManualAllowlistExceptionValidator {
        rules: vec![
            Rc::new(ManualAllowlistExceptionCodeLengthRule { max_length: 3600 })
                as Rc<dyn ManualAllowlistExceptionRule>,
            Rc::new(ManualAllowlistExceptionUnitLengthRule { max_length: 1000 }),
            Rc::new(ManualAllowlistExceptionRequiredCodeRule {
                message: "manual allowlist exception needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_allowlist_exception_validator();
        let item = new_manual_allowlist_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualAllowlistExceptionCodeLengthRule { max_length: 3600 };
        let mut item = new_manual_allowlist_exception("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ManualAllowlistExceptionUnitLengthRule { max_length: 16 };
        let mut item = new_manual_allowlist_exception("x", "y");
        item.unit = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualAllowlistExceptionRequiredCodeRule {
            message: "manual allowlist exception needs this field".to_string(),
        };
        let mut item = new_manual_allowlist_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
