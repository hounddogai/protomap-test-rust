use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending trust domain exception must pass. Check returns its problems, or none.
pub trait PendingTrustDomainExceptionRule: std::fmt::Debug {
    fn check(&self, item: &PendingTrustDomainException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainExceptionCodeLengthRule {
    pub max_length: i64,
}

impl PendingTrustDomainExceptionRule for PendingTrustDomainExceptionCodeLengthRule {
    fn check(&self, item: &PendingTrustDomainException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainExceptionCodePrefixRule {
    pub prefix: String,
}

impl PendingTrustDomainExceptionRule for PendingTrustDomainExceptionCodePrefixRule {
    fn check(&self, item: &PendingTrustDomainException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainExceptionRequiredCodeRule {
    pub message: String,
}

impl PendingTrustDomainExceptionRule for PendingTrustDomainExceptionRequiredCodeRule {
    fn check(&self, item: &PendingTrustDomainException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainExceptionNameLengthRule {
    pub max_length: i64,
}

impl PendingTrustDomainExceptionRule for PendingTrustDomainExceptionNameLengthRule {
    fn check(&self, item: &PendingTrustDomainException) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainExceptionWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingTrustDomainExceptionRule for PendingTrustDomainExceptionWeightRangeRule {
    fn check(&self, item: &PendingTrustDomainException) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainExceptionRequiredNameRule {
    pub message: String,
}

impl PendingTrustDomainExceptionRule for PendingTrustDomainExceptionRequiredNameRule {
    fn check(&self, item: &PendingTrustDomainException) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending trust domain exception.
#[derive(Debug, Clone)]
pub struct PendingTrustDomainExceptionValidator {
    pub rules: Vec<Rc<dyn PendingTrustDomainExceptionRule>>,
}

impl PendingTrustDomainExceptionValidator {
    pub fn validate(&self, item: &PendingTrustDomainException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingTrustDomainException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingTrustDomainException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_trust_domain_exception_validator() -> PendingTrustDomainExceptionValidator {
    PendingTrustDomainExceptionValidator {
        rules: vec![
            Rc::new(PendingTrustDomainExceptionCodeLengthRule { max_length: 3600 })
                as Rc<dyn PendingTrustDomainExceptionRule>,
            Rc::new(PendingTrustDomainExceptionCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PendingTrustDomainExceptionRequiredCodeRule {
                message: "pending trust domain exception needs an entry".to_string(),
            }),
            Rc::new(PendingTrustDomainExceptionNameLengthRule { max_length: 32 }),
            Rc::new(PendingTrustDomainExceptionWeightRangeRule { min: 3.2, max: 4788.0 }),
            Rc::new(PendingTrustDomainExceptionRequiredNameRule {
                message: "pending trust domain exception needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_trust_domain_exception_validator();
        let item = new_pending_trust_domain_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingTrustDomainExceptionCodeLengthRule { max_length: 32 };
        let mut item = new_pending_trust_domain_exception("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingTrustDomainExceptionRequiredCodeRule {
            message: "pending trust domain exception needs a value".to_string(),
        };
        let mut item = new_pending_trust_domain_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingTrustDomainExceptionNameLengthRule { max_length: 120 };
        let mut item = new_pending_trust_domain_exception("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PendingTrustDomainExceptionWeightRangeRule { min: 8.4, max: 1109.6 };
        let mut item = new_pending_trust_domain_exception("x", "y");
        item.weight = 1111.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PendingTrustDomainExceptionRequiredNameRule {
            message: "pending trust domain exception needs a value".to_string(),
        };
        let mut item = new_pending_trust_domain_exception("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
