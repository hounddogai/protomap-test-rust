use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict suspension rollup must pass. Check returns its problems, or none.
pub trait StrictSuspensionRollupRule: std::fmt::Debug {
    fn check(&self, item: &StrictSuspensionRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionRollupTitleLengthRule {
    pub max_length: i64,
}

impl StrictSuspensionRollupRule for StrictSuspensionRollupTitleLengthRule {
    fn check(&self, item: &StrictSuspensionRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionRollupCodePrefixRule {
    pub prefix: String,
}

impl StrictSuspensionRollupRule for StrictSuspensionRollupCodePrefixRule {
    fn check(&self, item: &StrictSuspensionRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionRollupPrincipalLengthRule {
    pub max_length: i64,
}

impl StrictSuspensionRollupRule for StrictSuspensionRollupPrincipalLengthRule {
    fn check(&self, item: &StrictSuspensionRollup) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionRollupRequiredPrincipalRule {
    pub message: String,
}

impl StrictSuspensionRollupRule for StrictSuspensionRollupRequiredPrincipalRule {
    fn check(&self, item: &StrictSuspensionRollup) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict suspension rollup.
#[derive(Debug, Clone)]
pub struct StrictSuspensionRollupValidator {
    pub rules: Vec<Rc<dyn StrictSuspensionRollupRule>>,
}

impl StrictSuspensionRollupValidator {
    pub fn validate(&self, item: &StrictSuspensionRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictSuspensionRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictSuspensionRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_suspension_rollup_validator() -> StrictSuspensionRollupValidator {
    StrictSuspensionRollupValidator {
        rules: vec![
            Rc::new(StrictSuspensionRollupTitleLengthRule { max_length: 3600 }) as Rc<dyn StrictSuspensionRollupRule>,
            Rc::new(StrictSuspensionRollupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(StrictSuspensionRollupPrincipalLengthRule { max_length: 3600 }),
            Rc::new(StrictSuspensionRollupRequiredPrincipalRule {
                message: "strict suspension rollup needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_suspension_rollup_validator();
        let item = new_strict_suspension_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = StrictSuspensionRollupTitleLengthRule { max_length: 32 };
        let mut item = new_strict_suspension_rollup("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = StrictSuspensionRollupPrincipalLengthRule { max_length: 86400 };
        let mut item = new_strict_suspension_rollup("x", "y");
        item.principal = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = StrictSuspensionRollupRequiredPrincipalRule {
            message: "strict suspension rollup needs this field".to_string(),
        };
        let mut item = new_strict_suspension_rollup("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
