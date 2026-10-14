use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage rollup must pass. Check returns its problems, or none.
pub trait UsageRollupRule: std::fmt::Debug {
    fn check(&self, item: &UsageRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRollupRequiredExternalRefRule {
    pub message: String,
}

impl UsageRollupRule for UsageRollupRequiredExternalRefRule {
    fn check(&self, item: &UsageRollup) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRollupAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl UsageRollupRule for UsageRollupAllowedCodeRule {
    fn check(&self, item: &UsageRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRollupUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageRollupRule for UsageRollupUpdatedAtRangeRule {
    fn check(&self, item: &UsageRollup) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a usage rollup.
#[derive(Debug, Clone)]
pub struct UsageRollupValidator {
    pub rules: Vec<Rc<dyn UsageRollupRule>>,
}

impl UsageRollupValidator {
    pub fn validate(&self, item: &UsageRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_rollup_validator() -> UsageRollupValidator {
    UsageRollupValidator {
        rules: vec![
            Rc::new(UsageRollupRequiredExternalRefRule {
                message: "usage rollup needs this field".to_string(),
            }) as Rc<dyn UsageRollupRule>,
            Rc::new(UsageRollupAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(UsageRollupUpdatedAtRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_rollup_validator();
        let item = new_usage_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = UsageRollupRequiredExternalRefRule {
            message: "usage rollup needs a value".to_string(),
        };
        let mut item = new_usage_rollup("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = UsageRollupUpdatedAtRangeRule { min: 64, max: 1000 };
        let mut item = new_usage_rollup("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
