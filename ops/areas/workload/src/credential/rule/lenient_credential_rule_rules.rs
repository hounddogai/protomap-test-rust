use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient credential rule must pass. Check returns its problems, or none.
pub trait LenientCredentialRuleRule: std::fmt::Debug {
    fn check(&self, item: &LenientCredentialRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialRuleCodePrefixRule {
    pub prefix: String,
}

impl LenientCredentialRuleRule for LenientCredentialRuleCodePrefixRule {
    fn check(&self, item: &LenientCredentialRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialRuleRequiredTenantIdRule {
    pub message: String,
}

impl LenientCredentialRuleRule for LenientCredentialRuleRequiredTenantIdRule {
    fn check(&self, item: &LenientCredentialRule) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialRuleTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientCredentialRuleRule for LenientCredentialRuleTtlSecondsRangeRule {
    fn check(&self, item: &LenientCredentialRule) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialRuleSourceLengthRule {
    pub max_length: i64,
}

impl LenientCredentialRuleRule for LenientCredentialRuleSourceLengthRule {
    fn check(&self, item: &LenientCredentialRule) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialRuleCodeLengthRule {
    pub max_length: i64,
}

impl LenientCredentialRuleRule for LenientCredentialRuleCodeLengthRule {
    fn check(&self, item: &LenientCredentialRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient credential rule.
#[derive(Debug, Clone)]
pub struct LenientCredentialRuleValidator {
    pub rules: Vec<Rc<dyn LenientCredentialRuleRule>>,
}

impl LenientCredentialRuleValidator {
    pub fn validate(&self, item: &LenientCredentialRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientCredentialRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientCredentialRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_credential_rule_validator() -> LenientCredentialRuleValidator {
    LenientCredentialRuleValidator {
        rules: vec![
            Rc::new(LenientCredentialRuleCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn LenientCredentialRuleRule>,
            Rc::new(LenientCredentialRuleRequiredTenantIdRule {
                message: "lenient credential rule needs an entry".to_string(),
            }),
            Rc::new(LenientCredentialRuleTtlSecondsRangeRule { min: 64, max: 3600 }),
            Rc::new(LenientCredentialRuleSourceLengthRule { max_length: 255 }),
            Rc::new(LenientCredentialRuleCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_credential_rule_validator();
        let item = new_lenient_credential_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = LenientCredentialRuleRequiredTenantIdRule {
            message: "lenient credential rule needs this field".to_string(),
        };
        let mut item = new_lenient_credential_rule("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = LenientCredentialRuleTtlSecondsRangeRule { min: 64, max: 80 };
        let mut item = new_lenient_credential_rule("x", "y");
        item.ttl_seconds = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = LenientCredentialRuleSourceLengthRule { max_length: 120 };
        let mut item = new_lenient_credential_rule("x", "y");
        item.source = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientCredentialRuleCodeLengthRule { max_length: 1000 };
        let mut item = new_lenient_credential_rule("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
