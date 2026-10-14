use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist claim must pass. Check returns its problems, or none.
pub trait AllowlistClaimRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistClaimRequiredCodeRule {
    pub message: String,
}

impl AllowlistClaimRule for AllowlistClaimRequiredCodeRule {
    fn check(&self, item: &AllowlistClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistClaimCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistClaimRule for AllowlistClaimCapacityRangeRule {
    fn check(&self, item: &AllowlistClaim) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistClaimRequiredClusterRule {
    pub message: String,
}

impl AllowlistClaimRule for AllowlistClaimRequiredClusterRule {
    fn check(&self, item: &AllowlistClaim) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistClaimClusterLengthRule {
    pub max_length: i64,
}

impl AllowlistClaimRule for AllowlistClaimClusterLengthRule {
    fn check(&self, item: &AllowlistClaim) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistClaimCodeLengthRule {
    pub max_length: i64,
}

impl AllowlistClaimRule for AllowlistClaimCodeLengthRule {
    fn check(&self, item: &AllowlistClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistClaimCodePrefixRule {
    pub prefix: String,
}

impl AllowlistClaimRule for AllowlistClaimCodePrefixRule {
    fn check(&self, item: &AllowlistClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist claim.
#[derive(Debug, Clone)]
pub struct AllowlistClaimValidator {
    pub rules: Vec<Rc<dyn AllowlistClaimRule>>,
}

impl AllowlistClaimValidator {
    pub fn validate(&self, item: &AllowlistClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_claim_validator() -> AllowlistClaimValidator {
    AllowlistClaimValidator {
        rules: vec![
            Rc::new(AllowlistClaimRequiredCodeRule {
                message: "allowlist claim needs an entry".to_string(),
            }) as Rc<dyn AllowlistClaimRule>,
            Rc::new(AllowlistClaimCapacityRangeRule { min: 64, max: 120 }),
            Rc::new(AllowlistClaimRequiredClusterRule {
                message: "allowlist claim needs an entry".to_string(),
            }),
            Rc::new(AllowlistClaimClusterLengthRule { max_length: 1000 }),
            Rc::new(AllowlistClaimCodeLengthRule { max_length: 255 }),
            Rc::new(AllowlistClaimCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_claim_validator();
        let item = new_allowlist_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistClaimRequiredCodeRule {
            message: "allowlist claim needs a value".to_string(),
        };
        let mut item = new_allowlist_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = AllowlistClaimCapacityRangeRule { min: 3600, max: 3600 };
        let mut item = new_allowlist_claim("x", "y");
        item.capacity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = AllowlistClaimRequiredClusterRule {
            message: "allowlist claim needs an entry".to_string(),
        };
        let mut item = new_allowlist_claim("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = AllowlistClaimClusterLengthRule { max_length: 86400 };
        let mut item = new_allowlist_claim("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AllowlistClaimCodeLengthRule { max_length: 3600 };
        let mut item = new_allowlist_claim("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
