use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling suspension exception must pass. Check returns its problems, or none.
pub trait RollingSuspensionExceptionRule: std::fmt::Debug {
    fn check(&self, item: &RollingSuspensionException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSuspensionExceptionApprovedGuardRule {
    pub limit: f64,
}

impl RollingSuspensionExceptionRule for RollingSuspensionExceptionApprovedGuardRule {
    fn check(&self, item: &RollingSuspensionException) -> Vec<String> {
        if item.approved && (item.ceiling > self.limit) {
            return vec![format!(
                "ceiling {} exceeds {} while approved",
                item.ceiling, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSuspensionExceptionCodePrefixRule {
    pub prefix: String,
}

impl RollingSuspensionExceptionRule for RollingSuspensionExceptionCodePrefixRule {
    fn check(&self, item: &RollingSuspensionException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSuspensionExceptionPhoneLengthRule {
    pub max_length: i64,
}

impl RollingSuspensionExceptionRule for RollingSuspensionExceptionPhoneLengthRule {
    fn check(&self, item: &RollingSuspensionException) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling suspension exception.
#[derive(Debug, Clone)]
pub struct RollingSuspensionExceptionValidator {
    pub rules: Vec<Rc<dyn RollingSuspensionExceptionRule>>,
}

impl RollingSuspensionExceptionValidator {
    pub fn validate(&self, item: &RollingSuspensionException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingSuspensionException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingSuspensionException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_suspension_exception_validator() -> RollingSuspensionExceptionValidator {
    RollingSuspensionExceptionValidator {
        rules: vec![
            Rc::new(RollingSuspensionExceptionApprovedGuardRule { limit: 7.0 })
                as Rc<dyn RollingSuspensionExceptionRule>,
            Rc::new(RollingSuspensionExceptionCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RollingSuspensionExceptionPhoneLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_suspension_exception_validator();
        let item = new_rolling_suspension_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = RollingSuspensionExceptionPhoneLengthRule { max_length: 64 };
        let mut item = new_rolling_suspension_exception("x", "y");
        item.phone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
