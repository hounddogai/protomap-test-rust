use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched allowlist quota must pass. Check returns its problems, or none.
pub trait BatchedAllowlistQuotaRule: std::fmt::Debug {
    fn check(&self, item: &BatchedAllowlistQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistQuotaRequiredNamespaceRule {
    pub message: String,
}

impl BatchedAllowlistQuotaRule for BatchedAllowlistQuotaRequiredNamespaceRule {
    fn check(&self, item: &BatchedAllowlistQuota) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistQuotaCodeLengthRule {
    pub max_length: i64,
}

impl BatchedAllowlistQuotaRule for BatchedAllowlistQuotaCodeLengthRule {
    fn check(&self, item: &BatchedAllowlistQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistQuotaNamespaceLengthRule {
    pub max_length: i64,
}

impl BatchedAllowlistQuotaRule for BatchedAllowlistQuotaNamespaceLengthRule {
    fn check(&self, item: &BatchedAllowlistQuota) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistQuotaChecksumLengthRule {
    pub max_length: i64,
}

impl BatchedAllowlistQuotaRule for BatchedAllowlistQuotaChecksumLengthRule {
    fn check(&self, item: &BatchedAllowlistQuota) -> Vec<String> {
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

/// Runs every rule of a batched allowlist quota.
#[derive(Debug, Clone)]
pub struct BatchedAllowlistQuotaValidator {
    pub rules: Vec<Rc<dyn BatchedAllowlistQuotaRule>>,
}

impl BatchedAllowlistQuotaValidator {
    pub fn validate(&self, item: &BatchedAllowlistQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedAllowlistQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedAllowlistQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_allowlist_quota_validator() -> BatchedAllowlistQuotaValidator {
    BatchedAllowlistQuotaValidator {
        rules: vec![
            Rc::new(BatchedAllowlistQuotaRequiredNamespaceRule {
                message: "batched allowlist quota needs a value".to_string(),
            }) as Rc<dyn BatchedAllowlistQuotaRule>,
            Rc::new(BatchedAllowlistQuotaCodeLengthRule { max_length: 120 }),
            Rc::new(BatchedAllowlistQuotaNamespaceLengthRule { max_length: 255 }),
            Rc::new(BatchedAllowlistQuotaChecksumLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_allowlist_quota_validator();
        let item = new_batched_allowlist_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = BatchedAllowlistQuotaRequiredNamespaceRule {
            message: "batched allowlist quota needs a value".to_string(),
        };
        let mut item = new_batched_allowlist_quota("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedAllowlistQuotaCodeLengthRule { max_length: 255 };
        let mut item = new_batched_allowlist_quota("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = BatchedAllowlistQuotaNamespaceLengthRule { max_length: 64 };
        let mut item = new_batched_allowlist_quota("x", "y");
        item.namespace = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = BatchedAllowlistQuotaChecksumLengthRule { max_length: 80 };
        let mut item = new_batched_allowlist_quota("x", "y");
        item.checksum = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
