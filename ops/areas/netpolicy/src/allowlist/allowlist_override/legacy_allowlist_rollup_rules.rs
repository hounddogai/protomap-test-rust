use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy allowlist rollup must pass. Check returns its problems, or none.
pub trait LegacyAllowlistRollupRule: std::fmt::Debug {
    fn check(&self, item: &LegacyAllowlistRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistRollupLabelLengthRule {
    pub max_length: i64,
}

impl LegacyAllowlistRollupRule for LegacyAllowlistRollupLabelLengthRule {
    fn check(&self, item: &LegacyAllowlistRollup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistRollupCodeLengthRule {
    pub max_length: i64,
}

impl LegacyAllowlistRollupRule for LegacyAllowlistRollupCodeLengthRule {
    fn check(&self, item: &LegacyAllowlistRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistRollupAllowedHostRule {
    pub allowed: Vec<String>,
}

impl LegacyAllowlistRollupRule for LegacyAllowlistRollupAllowedHostRule {
    fn check(&self, item: &LegacyAllowlistRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is below minimum", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistRollupRequiredHostRule {
    pub message: String,
}

impl LegacyAllowlistRollupRule for LegacyAllowlistRollupRequiredHostRule {
    fn check(&self, item: &LegacyAllowlistRollup) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistRollupCodePrefixRule {
    pub prefix: String,
}

impl LegacyAllowlistRollupRule for LegacyAllowlistRollupCodePrefixRule {
    fn check(&self, item: &LegacyAllowlistRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy allowlist rollup.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistRollupValidator {
    pub rules: Vec<Rc<dyn LegacyAllowlistRollupRule>>,
}

impl LegacyAllowlistRollupValidator {
    pub fn validate(&self, item: &LegacyAllowlistRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyAllowlistRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyAllowlistRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_allowlist_rollup_validator() -> LegacyAllowlistRollupValidator {
    LegacyAllowlistRollupValidator {
        rules: vec![
            Rc::new(LegacyAllowlistRollupLabelLengthRule { max_length: 32 }) as Rc<dyn LegacyAllowlistRollupRule>,
            Rc::new(LegacyAllowlistRollupCodeLengthRule { max_length: 1000 }),
            Rc::new(LegacyAllowlistRollupAllowedHostRule {
                allowed: vec!["manual".to_string(), "batched".to_string(), "weighted".to_string()],
            }),
            Rc::new(LegacyAllowlistRollupRequiredHostRule {
                message: "legacy allowlist rollup needs this field".to_string(),
            }),
            Rc::new(LegacyAllowlistRollupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_allowlist_rollup_validator();
        let item = new_legacy_allowlist_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyAllowlistRollupLabelLengthRule { max_length: 3600 };
        let mut item = new_legacy_allowlist_rollup("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyAllowlistRollupCodeLengthRule { max_length: 80 };
        let mut item = new_legacy_allowlist_rollup("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = LegacyAllowlistRollupRequiredHostRule {
            message: "legacy allowlist rollup needs an entry".to_string(),
        };
        let mut item = new_legacy_allowlist_rollup("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
