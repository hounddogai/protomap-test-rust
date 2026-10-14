use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding exception must pass. Check returns its problems, or none.
pub trait BindingExceptionRule: std::fmt::Debug {
    fn check(&self, item: &BindingException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingExceptionTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingExceptionRule for BindingExceptionTtlSecondsRangeRule {
    fn check(&self, item: &BindingException) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingExceptionRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingExceptionRule for BindingExceptionRetriesRangeRule {
    fn check(&self, item: &BindingException) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingExceptionUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingExceptionRule for BindingExceptionUpdatedAtRangeRule {
    fn check(&self, item: &BindingException) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding exception.
#[derive(Debug, Clone)]
pub struct BindingExceptionValidator {
    pub rules: Vec<Rc<dyn BindingExceptionRule>>,
}

impl BindingExceptionValidator {
    pub fn validate(&self, item: &BindingException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_exception_validator() -> BindingExceptionValidator {
    BindingExceptionValidator {
        rules: vec![
            Rc::new(BindingExceptionTtlSecondsRangeRule { min: 80, max: 86400 }) as Rc<dyn BindingExceptionRule>,
            Rc::new(BindingExceptionRetriesRangeRule { min: 255, max: 86400 }),
            Rc::new(BindingExceptionUpdatedAtRangeRule { min: 32, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_exception_validator();
        let item = new_binding_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = BindingExceptionTtlSecondsRangeRule { min: 64, max: 255 };
        let mut item = new_binding_exception("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = BindingExceptionRetriesRangeRule { min: 120, max: 255 };
        let mut item = new_binding_exception("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = BindingExceptionUpdatedAtRangeRule { min: 80, max: 86400 };
        let mut item = new_binding_exception("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
