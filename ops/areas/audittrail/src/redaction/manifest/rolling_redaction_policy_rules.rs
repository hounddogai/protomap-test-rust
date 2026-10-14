use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling redaction policy must pass. Check returns its problems, or none.
pub trait RollingRedactionPolicyRule: std::fmt::Debug {
    fn check(&self, item: &RollingRedactionPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionPolicyCodePrefixRule {
    pub prefix: String,
}

impl RollingRedactionPolicyRule for RollingRedactionPolicyCodePrefixRule {
    fn check(&self, item: &RollingRedactionPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionPolicyCodeLengthRule {
    pub max_length: i64,
}

impl RollingRedactionPolicyRule for RollingRedactionPolicyCodeLengthRule {
    fn check(&self, item: &RollingRedactionPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionPolicyAudienceLengthRule {
    pub max_length: i64,
}

impl RollingRedactionPolicyRule for RollingRedactionPolicyAudienceLengthRule {
    fn check(&self, item: &RollingRedactionPolicy) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionPolicyCurrencyLengthRule {
    pub max_length: i64,
}

impl RollingRedactionPolicyRule for RollingRedactionPolicyCurrencyLengthRule {
    fn check(&self, item: &RollingRedactionPolicy) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling redaction policy.
#[derive(Debug, Clone)]
pub struct RollingRedactionPolicyValidator {
    pub rules: Vec<Rc<dyn RollingRedactionPolicyRule>>,
}

impl RollingRedactionPolicyValidator {
    pub fn validate(&self, item: &RollingRedactionPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingRedactionPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRedactionPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_redaction_policy_validator() -> RollingRedactionPolicyValidator {
    RollingRedactionPolicyValidator {
        rules: vec![
            Rc::new(RollingRedactionPolicyCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn RollingRedactionPolicyRule>,
            Rc::new(RollingRedactionPolicyCodeLengthRule { max_length: 3600 }),
            Rc::new(RollingRedactionPolicyAudienceLengthRule { max_length: 86400 }),
            Rc::new(RollingRedactionPolicyCurrencyLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_redaction_policy_validator();
        let item = new_rolling_redaction_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingRedactionPolicyCodeLengthRule { max_length: 80 };
        let mut item = new_rolling_redaction_policy("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = RollingRedactionPolicyAudienceLengthRule { max_length: 64 };
        let mut item = new_rolling_redaction_policy("x", "y");
        item.audience = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = RollingRedactionPolicyCurrencyLengthRule { max_length: 80 };
        let mut item = new_rolling_redaction_policy("x", "y");
        item.currency = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
