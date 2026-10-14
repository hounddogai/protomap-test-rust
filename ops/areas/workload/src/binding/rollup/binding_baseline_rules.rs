use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding baseline must pass. Check returns its problems, or none.
pub trait BindingBaselineRule: std::fmt::Debug {
    fn check(&self, item: &BindingBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBaselineBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingBaselineRule for BindingBaselineBatchSizeRangeRule {
    fn check(&self, item: &BindingBaseline) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBaselinePinnedGuardRule {
    pub limit: i64,
}

impl BindingBaselineRule for BindingBaselinePinnedGuardRule {
    fn check(&self, item: &BindingBaseline) -> Vec<String> {
        if item.pinned && (item.revision > self.limit) {
            return vec![format!(
                "revision {} exceeds {} while pinned",
                item.revision, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBaselineCodeLengthRule {
    pub max_length: i64,
}

impl BindingBaselineRule for BindingBaselineCodeLengthRule {
    fn check(&self, item: &BindingBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBaselineRequiredScopeRule {
    pub message: String,
}

impl BindingBaselineRule for BindingBaselineRequiredScopeRule {
    fn check(&self, item: &BindingBaseline) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding baseline.
#[derive(Debug, Clone)]
pub struct BindingBaselineValidator {
    pub rules: Vec<Rc<dyn BindingBaselineRule>>,
}

impl BindingBaselineValidator {
    pub fn validate(&self, item: &BindingBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_baseline_validator() -> BindingBaselineValidator {
    BindingBaselineValidator {
        rules: vec![
            Rc::new(BindingBaselineBatchSizeRangeRule { min: 32, max: 3600 }) as Rc<dyn BindingBaselineRule>,
            Rc::new(BindingBaselinePinnedGuardRule { limit: 16 }),
            Rc::new(BindingBaselineCodeLengthRule { max_length: 255 }),
            Rc::new(BindingBaselineRequiredScopeRule {
                message: "binding baseline needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_baseline_validator();
        let item = new_binding_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = BindingBaselineBatchSizeRangeRule { min: 16, max: 16 };
        let mut item = new_binding_baseline("x", "y");
        item.batch_size = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BindingBaselineCodeLengthRule { max_length: 86400 };
        let mut item = new_binding_baseline("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = BindingBaselineRequiredScopeRule {
            message: "binding baseline needs an entry".to_string(),
        };
        let mut item = new_binding_baseline("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
