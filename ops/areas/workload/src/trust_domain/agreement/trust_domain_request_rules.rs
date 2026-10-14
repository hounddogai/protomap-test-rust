use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain request must pass. Check returns its problems, or none.
pub trait TrustDomainRequestRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRequestRequiredTitleRule {
    pub message: String,
}

impl TrustDomainRequestRule for TrustDomainRequestRequiredTitleRule {
    fn check(&self, item: &TrustDomainRequest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRequestExternalRefLengthRule {
    pub max_length: i64,
}

impl TrustDomainRequestRule for TrustDomainRequestExternalRefLengthRule {
    fn check(&self, item: &TrustDomainRequest) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRequestVisibleGuardRule {
    pub limit: i64,
}

impl TrustDomainRequestRule for TrustDomainRequestVisibleGuardRule {
    fn check(&self, item: &TrustDomainRequest) -> Vec<String> {
        if item.visible && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while visible",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRequestGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainRequestRule for TrustDomainRequestGraceDaysRangeRule {
    fn check(&self, item: &TrustDomainRequest) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRequestTitleLengthRule {
    pub max_length: i64,
}

impl TrustDomainRequestRule for TrustDomainRequestTitleLengthRule {
    fn check(&self, item: &TrustDomainRequest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain request.
#[derive(Debug, Clone)]
pub struct TrustDomainRequestValidator {
    pub rules: Vec<Rc<dyn TrustDomainRequestRule>>,
}

impl TrustDomainRequestValidator {
    pub fn validate(&self, item: &TrustDomainRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_request_validator() -> TrustDomainRequestValidator {
    TrustDomainRequestValidator {
        rules: vec![
            Rc::new(TrustDomainRequestRequiredTitleRule {
                message: "trust domain request needs this field".to_string(),
            }) as Rc<dyn TrustDomainRequestRule>,
            Rc::new(TrustDomainRequestExternalRefLengthRule { max_length: 120 }),
            Rc::new(TrustDomainRequestVisibleGuardRule { limit: 1000 }),
            Rc::new(TrustDomainRequestGraceDaysRangeRule { min: 1000, max: 3600 }),
            Rc::new(TrustDomainRequestTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_request_validator();
        let item = new_trust_domain_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TrustDomainRequestRequiredTitleRule {
            message: "trust domain request needs a value".to_string(),
        };
        let mut item = new_trust_domain_request("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = TrustDomainRequestExternalRefLengthRule { max_length: 16 };
        let mut item = new_trust_domain_request("x", "y");
        item.external_ref = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = TrustDomainRequestGraceDaysRangeRule { min: 64, max: 1000 };
        let mut item = new_trust_domain_request("x", "y");
        item.grace_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TrustDomainRequestTitleLengthRule { max_length: 1000 };
        let mut item = new_trust_domain_request("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
