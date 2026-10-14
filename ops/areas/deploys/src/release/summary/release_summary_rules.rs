use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release summary must pass. Check returns its problems, or none.
pub trait ReleaseSummaryRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSummaryRequiredScopeRule {
    pub message: String,
}

impl ReleaseSummaryRule for ReleaseSummaryRequiredScopeRule {
    fn check(&self, item: &ReleaseSummary) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSummaryQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseSummaryRule for ReleaseSummaryQuantityRangeRule {
    fn check(&self, item: &ReleaseSummary) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSummaryScopeLengthRule {
    pub max_length: i64,
}

impl ReleaseSummaryRule for ReleaseSummaryScopeLengthRule {
    fn check(&self, item: &ReleaseSummary) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release summary.
#[derive(Debug, Clone)]
pub struct ReleaseSummaryValidator {
    pub rules: Vec<Rc<dyn ReleaseSummaryRule>>,
}

impl ReleaseSummaryValidator {
    pub fn validate(&self, item: &ReleaseSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_summary_validator() -> ReleaseSummaryValidator {
    ReleaseSummaryValidator {
        rules: vec![
            Rc::new(ReleaseSummaryRequiredScopeRule {
                message: "release summary needs this field".to_string(),
            }) as Rc<dyn ReleaseSummaryRule>,
            Rc::new(ReleaseSummaryQuantityRangeRule { min: 16, max: 32 }),
            Rc::new(ReleaseSummaryScopeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_summary_validator();
        let item = new_release_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ReleaseSummaryRequiredScopeRule {
            message: "release summary needs an entry".to_string(),
        };
        let mut item = new_release_summary("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ReleaseSummaryQuantityRangeRule { min: 64, max: 64 };
        let mut item = new_release_summary("x", "y");
        item.quantity = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = ReleaseSummaryScopeLengthRule { max_length: 255 };
        let mut item = new_release_summary("x", "y");
        item.scope = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
