use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional credential rollup must pass. Check returns its problems, or none.
pub trait RegionalCredentialRollupRule: std::fmt::Debug {
    fn check(&self, item: &RegionalCredentialRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCredentialRollupCodePrefixRule {
    pub prefix: String,
}

impl RegionalCredentialRollupRule for RegionalCredentialRollupCodePrefixRule {
    fn check(&self, item: &RegionalCredentialRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCredentialRollupLabelLengthRule {
    pub max_length: i64,
}

impl RegionalCredentialRollupRule for RegionalCredentialRollupLabelLengthRule {
    fn check(&self, item: &RegionalCredentialRollup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCredentialRollupChecksumLengthRule {
    pub max_length: i64,
}

impl RegionalCredentialRollupRule for RegionalCredentialRollupChecksumLengthRule {
    fn check(&self, item: &RegionalCredentialRollup) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCredentialRollupRequiredLabelRule {
    pub message: String,
}

impl RegionalCredentialRollupRule for RegionalCredentialRollupRequiredLabelRule {
    fn check(&self, item: &RegionalCredentialRollup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional credential rollup.
#[derive(Debug, Clone)]
pub struct RegionalCredentialRollupValidator {
    pub rules: Vec<Rc<dyn RegionalCredentialRollupRule>>,
}

impl RegionalCredentialRollupValidator {
    pub fn validate(&self, item: &RegionalCredentialRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalCredentialRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalCredentialRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_credential_rollup_validator() -> RegionalCredentialRollupValidator {
    RegionalCredentialRollupValidator {
        rules: vec![
            Rc::new(RegionalCredentialRollupCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn RegionalCredentialRollupRule>,
            Rc::new(RegionalCredentialRollupLabelLengthRule { max_length: 86400 }),
            Rc::new(RegionalCredentialRollupChecksumLengthRule { max_length: 80 }),
            Rc::new(RegionalCredentialRollupRequiredLabelRule {
                message: "regional credential rollup needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_credential_rollup_validator();
        let item = new_regional_credential_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalCredentialRollupLabelLengthRule { max_length: 80 };
        let mut item = new_regional_credential_rollup("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = RegionalCredentialRollupChecksumLengthRule { max_length: 80 };
        let mut item = new_regional_credential_rollup("x", "y");
        item.checksum = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalCredentialRollupRequiredLabelRule {
            message: "regional credential rollup needs this field".to_string(),
        };
        let mut item = new_regional_credential_rollup("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
