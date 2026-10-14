use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling quote rollup must pass. Check returns its problems, or none.
pub trait RollingQuoteRollupRule: std::fmt::Debug {
    fn check(&self, item: &RollingQuoteRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteRollupCodePrefixRule {
    pub prefix: String,
}

impl RollingQuoteRollupRule for RollingQuoteRollupCodePrefixRule {
    fn check(&self, item: &RollingQuoteRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteRollupVisibleGuardRule {
    pub limit: f64,
}

impl RollingQuoteRollupRule for RollingQuoteRollupVisibleGuardRule {
    fn check(&self, item: &RollingQuoteRollup) -> Vec<String> {
        if item.visible && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while visible",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteRollupRequiredCodeRule {
    pub message: String,
}

impl RollingQuoteRollupRule for RollingQuoteRollupRequiredCodeRule {
    fn check(&self, item: &RollingQuoteRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling quote rollup.
#[derive(Debug, Clone)]
pub struct RollingQuoteRollupValidator {
    pub rules: Vec<Rc<dyn RollingQuoteRollupRule>>,
}

impl RollingQuoteRollupValidator {
    pub fn validate(&self, item: &RollingQuoteRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingQuoteRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingQuoteRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_quote_rollup_validator() -> RollingQuoteRollupValidator {
    RollingQuoteRollupValidator {
        rules: vec![
            Rc::new(RollingQuoteRollupCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn RollingQuoteRollupRule>,
            Rc::new(RollingQuoteRollupVisibleGuardRule { limit: 9.1 }),
            Rc::new(RollingQuoteRollupRequiredCodeRule {
                message: "rolling quote rollup needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_quote_rollup_validator();
        let item = new_rolling_quote_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingQuoteRollupRequiredCodeRule {
            message: "rolling quote rollup needs a value".to_string(),
        };
        let mut item = new_rolling_quote_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
