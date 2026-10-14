use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending nonce baseline must pass. Check returns its problems, or none.
pub trait PendingNonceBaselineRule: std::fmt::Debug {
    fn check(&self, item: &PendingNonceBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceBaselineCodePrefixRule {
    pub prefix: String,
}

impl PendingNonceBaselineRule for PendingNonceBaselineCodePrefixRule {
    fn check(&self, item: &PendingNonceBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceBaselineRequiredNameRule {
    pub message: String,
}

impl PendingNonceBaselineRule for PendingNonceBaselineRequiredNameRule {
    fn check(&self, item: &PendingNonceBaseline) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceBaselineCodeLengthRule {
    pub max_length: i64,
}

impl PendingNonceBaselineRule for PendingNonceBaselineCodeLengthRule {
    fn check(&self, item: &PendingNonceBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceBaselineNameLengthRule {
    pub max_length: i64,
}

impl PendingNonceBaselineRule for PendingNonceBaselineNameLengthRule {
    fn check(&self, item: &PendingNonceBaseline) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceBaselineClusterLengthRule {
    pub max_length: i64,
}

impl PendingNonceBaselineRule for PendingNonceBaselineClusterLengthRule {
    fn check(&self, item: &PendingNonceBaseline) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceBaselineRequiredCodeRule {
    pub message: String,
}

impl PendingNonceBaselineRule for PendingNonceBaselineRequiredCodeRule {
    fn check(&self, item: &PendingNonceBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending nonce baseline.
#[derive(Debug, Clone)]
pub struct PendingNonceBaselineValidator {
    pub rules: Vec<Rc<dyn PendingNonceBaselineRule>>,
}

impl PendingNonceBaselineValidator {
    pub fn validate(&self, item: &PendingNonceBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingNonceBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingNonceBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_nonce_baseline_validator() -> PendingNonceBaselineValidator {
    PendingNonceBaselineValidator {
        rules: vec![
            Rc::new(PendingNonceBaselineCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn PendingNonceBaselineRule>,
            Rc::new(PendingNonceBaselineRequiredNameRule {
                message: "pending nonce baseline needs this field".to_string(),
            }),
            Rc::new(PendingNonceBaselineCodeLengthRule { max_length: 16 }),
            Rc::new(PendingNonceBaselineNameLengthRule { max_length: 1000 }),
            Rc::new(PendingNonceBaselineClusterLengthRule { max_length: 3600 }),
            Rc::new(PendingNonceBaselineRequiredCodeRule {
                message: "pending nonce baseline needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_nonce_baseline_validator();
        let item = new_pending_nonce_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PendingNonceBaselineRequiredNameRule {
            message: "pending nonce baseline needs this field".to_string(),
        };
        let mut item = new_pending_nonce_baseline("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingNonceBaselineCodeLengthRule { max_length: 32 };
        let mut item = new_pending_nonce_baseline("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingNonceBaselineNameLengthRule { max_length: 16 };
        let mut item = new_pending_nonce_baseline("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = PendingNonceBaselineClusterLengthRule { max_length: 255 };
        let mut item = new_pending_nonce_baseline("x", "y");
        item.cluster = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingNonceBaselineRequiredCodeRule {
            message: "pending nonce baseline needs an entry".to_string(),
        };
        let mut item = new_pending_nonce_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
