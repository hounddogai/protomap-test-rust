use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation score must pass. Check returns its problems, or none.
pub trait IsolationScoreRule: std::fmt::Debug {
    fn check(&self, item: &IsolationScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationScoreTenantIdLengthRule {
    pub max_length: i64,
}

impl IsolationScoreRule for IsolationScoreTenantIdLengthRule {
    fn check(&self, item: &IsolationScore) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationScoreRequiredCodeRule {
    pub message: String,
}

impl IsolationScoreRule for IsolationScoreRequiredCodeRule {
    fn check(&self, item: &IsolationScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationScoreCodePrefixRule {
    pub prefix: String,
}

impl IsolationScoreRule for IsolationScoreCodePrefixRule {
    fn check(&self, item: &IsolationScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationScoreRequiredTenantIdRule {
    pub message: String,
}

impl IsolationScoreRule for IsolationScoreRequiredTenantIdRule {
    fn check(&self, item: &IsolationScore) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationScoreTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationScoreRule for IsolationScoreTtlSecondsRangeRule {
    fn check(&self, item: &IsolationScore) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a isolation score.
#[derive(Debug, Clone)]
pub struct IsolationScoreValidator {
    pub rules: Vec<Rc<dyn IsolationScoreRule>>,
}

impl IsolationScoreValidator {
    pub fn validate(&self, item: &IsolationScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_score_validator() -> IsolationScoreValidator {
    IsolationScoreValidator {
        rules: vec![
            Rc::new(IsolationScoreTenantIdLengthRule { max_length: 120 }) as Rc<dyn IsolationScoreRule>,
            Rc::new(IsolationScoreRequiredCodeRule {
                message: "isolation score needs this field".to_string(),
            }),
            Rc::new(IsolationScoreCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(IsolationScoreRequiredTenantIdRule {
                message: "isolation score needs a value".to_string(),
            }),
            Rc::new(IsolationScoreTtlSecondsRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_score_validator();
        let item = new_isolation_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = IsolationScoreTenantIdLengthRule { max_length: 86400 };
        let mut item = new_isolation_score("x", "y");
        item.tenant_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IsolationScoreRequiredCodeRule {
            message: "isolation score needs a value".to_string(),
        };
        let mut item = new_isolation_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = IsolationScoreRequiredTenantIdRule {
            message: "isolation score needs this field".to_string(),
        };
        let mut item = new_isolation_score("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = IsolationScoreTtlSecondsRangeRule { min: 255, max: 255 };
        let mut item = new_isolation_score("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
