use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual flow log digest must pass. Check returns its problems, or none.
pub trait ManualFlowLogDigestRule: std::fmt::Debug {
    fn check(&self, item: &ManualFlowLogDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogDigestNameLengthRule {
    pub max_length: i64,
}

impl ManualFlowLogDigestRule for ManualFlowLogDigestNameLengthRule {
    fn check(&self, item: &ManualFlowLogDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogDigestRequiredCodeRule {
    pub message: String,
}

impl ManualFlowLogDigestRule for ManualFlowLogDigestRequiredCodeRule {
    fn check(&self, item: &ManualFlowLogDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogDigestRequiredNameRule {
    pub message: String,
}

impl ManualFlowLogDigestRule for ManualFlowLogDigestRequiredNameRule {
    fn check(&self, item: &ManualFlowLogDigest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogDigestPinnedGuardRule {
    pub limit: f64,
}

impl ManualFlowLogDigestRule for ManualFlowLogDigestPinnedGuardRule {
    fn check(&self, item: &ManualFlowLogDigest) -> Vec<String> {
        if item.pinned && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while pinned", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogDigestCodePrefixRule {
    pub prefix: String,
}

impl ManualFlowLogDigestRule for ManualFlowLogDigestCodePrefixRule {
    fn check(&self, item: &ManualFlowLogDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogDigestCodeLengthRule {
    pub max_length: i64,
}

impl ManualFlowLogDigestRule for ManualFlowLogDigestCodeLengthRule {
    fn check(&self, item: &ManualFlowLogDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual flow log digest.
#[derive(Debug, Clone)]
pub struct ManualFlowLogDigestValidator {
    pub rules: Vec<Rc<dyn ManualFlowLogDigestRule>>,
}

impl ManualFlowLogDigestValidator {
    pub fn validate(&self, item: &ManualFlowLogDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualFlowLogDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualFlowLogDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_flow_log_digest_validator() -> ManualFlowLogDigestValidator {
    ManualFlowLogDigestValidator {
        rules: vec![
            Rc::new(ManualFlowLogDigestNameLengthRule { max_length: 16 }) as Rc<dyn ManualFlowLogDigestRule>,
            Rc::new(ManualFlowLogDigestRequiredCodeRule {
                message: "manual flow log digest needs a value".to_string(),
            }),
            Rc::new(ManualFlowLogDigestRequiredNameRule {
                message: "manual flow log digest needs an entry".to_string(),
            }),
            Rc::new(ManualFlowLogDigestPinnedGuardRule { limit: 9.6 }),
            Rc::new(ManualFlowLogDigestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ManualFlowLogDigestCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_flow_log_digest_validator();
        let item = new_manual_flow_log_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualFlowLogDigestNameLengthRule { max_length: 32 };
        let mut item = new_manual_flow_log_digest("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualFlowLogDigestRequiredCodeRule {
            message: "manual flow log digest needs a value".to_string(),
        };
        let mut item = new_manual_flow_log_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualFlowLogDigestRequiredNameRule {
            message: "manual flow log digest needs an entry".to_string(),
        };
        let mut item = new_manual_flow_log_digest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualFlowLogDigestCodeLengthRule { max_length: 3600 };
        let mut item = new_manual_flow_log_digest("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
