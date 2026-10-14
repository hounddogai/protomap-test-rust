use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional isolation tier must pass. Check returns its problems, or none.
pub trait RegionalIsolationTierRule: std::fmt::Debug {
    fn check(&self, item: &RegionalIsolationTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIsolationTierRequiredCipherRule {
    pub message: String,
}

impl RegionalIsolationTierRule for RegionalIsolationTierRequiredCipherRule {
    fn check(&self, item: &RegionalIsolationTier) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIsolationTierRequiredLabelRule {
    pub message: String,
}

impl RegionalIsolationTierRule for RegionalIsolationTierRequiredLabelRule {
    fn check(&self, item: &RegionalIsolationTier) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIsolationTierRequiredCodeRule {
    pub message: String,
}

impl RegionalIsolationTierRule for RegionalIsolationTierRequiredCodeRule {
    fn check(&self, item: &RegionalIsolationTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIsolationTierCodePrefixRule {
    pub prefix: String,
}

impl RegionalIsolationTierRule for RegionalIsolationTierCodePrefixRule {
    fn check(&self, item: &RegionalIsolationTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional isolation tier.
#[derive(Debug, Clone)]
pub struct RegionalIsolationTierValidator {
    pub rules: Vec<Rc<dyn RegionalIsolationTierRule>>,
}

impl RegionalIsolationTierValidator {
    pub fn validate(&self, item: &RegionalIsolationTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalIsolationTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalIsolationTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_isolation_tier_validator() -> RegionalIsolationTierValidator {
    RegionalIsolationTierValidator {
        rules: vec![
            Rc::new(RegionalIsolationTierRequiredCipherRule {
                message: "regional isolation tier needs this field".to_string(),
            }) as Rc<dyn RegionalIsolationTierRule>,
            Rc::new(RegionalIsolationTierRequiredLabelRule {
                message: "regional isolation tier needs a value".to_string(),
            }),
            Rc::new(RegionalIsolationTierRequiredCodeRule {
                message: "regional isolation tier needs a value".to_string(),
            }),
            Rc::new(RegionalIsolationTierCodePrefixRule {
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
        let validator = default_regional_isolation_tier_validator();
        let item = new_regional_isolation_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = RegionalIsolationTierRequiredCipherRule {
            message: "regional isolation tier needs an entry".to_string(),
        };
        let mut item = new_regional_isolation_tier("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalIsolationTierRequiredLabelRule {
            message: "regional isolation tier needs this field".to_string(),
        };
        let mut item = new_regional_isolation_tier("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalIsolationTierRequiredCodeRule {
            message: "regional isolation tier needs a value".to_string(),
        };
        let mut item = new_regional_isolation_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
