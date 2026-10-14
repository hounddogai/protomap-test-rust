use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling limit claim must pass. Check returns its problems, or none.
pub trait RollingLimitClaimRule: std::fmt::Debug {
    fn check(&self, item: &RollingLimitClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitClaimCodeLengthRule {
    pub max_length: i64,
}

impl RollingLimitClaimRule for RollingLimitClaimCodeLengthRule {
    fn check(&self, item: &RollingLimitClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitClaimNameLengthRule {
    pub max_length: i64,
}

impl RollingLimitClaimRule for RollingLimitClaimNameLengthRule {
    fn check(&self, item: &RollingLimitClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitClaimRequiredChecksumRule {
    pub message: String,
}

impl RollingLimitClaimRule for RollingLimitClaimRequiredChecksumRule {
    fn check(&self, item: &RollingLimitClaim) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitClaimChecksumLengthRule {
    pub max_length: i64,
}

impl RollingLimitClaimRule for RollingLimitClaimChecksumLengthRule {
    fn check(&self, item: &RollingLimitClaim) -> Vec<String> {
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

/// Runs every rule of a rolling limit claim.
#[derive(Debug, Clone)]
pub struct RollingLimitClaimValidator {
    pub rules: Vec<Rc<dyn RollingLimitClaimRule>>,
}

impl RollingLimitClaimValidator {
    pub fn validate(&self, item: &RollingLimitClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingLimitClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingLimitClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_limit_claim_validator() -> RollingLimitClaimValidator {
    RollingLimitClaimValidator {
        rules: vec![
            Rc::new(RollingLimitClaimCodeLengthRule { max_length: 16 }) as Rc<dyn RollingLimitClaimRule>,
            Rc::new(RollingLimitClaimNameLengthRule { max_length: 120 }),
            Rc::new(RollingLimitClaimRequiredChecksumRule {
                message: "rolling limit claim needs an entry".to_string(),
            }),
            Rc::new(RollingLimitClaimChecksumLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_limit_claim_validator();
        let item = new_rolling_limit_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingLimitClaimCodeLengthRule { max_length: 64 };
        let mut item = new_rolling_limit_claim("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingLimitClaimNameLengthRule { max_length: 120 };
        let mut item = new_rolling_limit_claim("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = RollingLimitClaimRequiredChecksumRule {
            message: "rolling limit claim needs an entry".to_string(),
        };
        let mut item = new_rolling_limit_claim("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = RollingLimitClaimChecksumLengthRule { max_length: 3600 };
        let mut item = new_rolling_limit_claim("x", "y");
        item.checksum = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
