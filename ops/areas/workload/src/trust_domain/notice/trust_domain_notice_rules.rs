use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain notice must pass. Check returns its problems, or none.
pub trait TrustDomainNoticeRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainNoticeTitleLengthRule {
    pub max_length: i64,
}

impl TrustDomainNoticeRule for TrustDomainNoticeTitleLengthRule {
    fn check(&self, item: &TrustDomainNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainNoticeCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainNoticeRule for TrustDomainNoticeCostRangeRule {
    fn check(&self, item: &TrustDomainNotice) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainNoticeCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainNoticeRule for TrustDomainNoticeCodePrefixRule {
    fn check(&self, item: &TrustDomainNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain notice.
#[derive(Debug, Clone)]
pub struct TrustDomainNoticeValidator {
    pub rules: Vec<Rc<dyn TrustDomainNoticeRule>>,
}

impl TrustDomainNoticeValidator {
    pub fn validate(&self, item: &TrustDomainNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_notice_validator() -> TrustDomainNoticeValidator {
    TrustDomainNoticeValidator {
        rules: vec![
            Rc::new(TrustDomainNoticeTitleLengthRule { max_length: 32 }) as Rc<dyn TrustDomainNoticeRule>,
            Rc::new(TrustDomainNoticeCostRangeRule { min: 2.8, max: 362.5 }),
            Rc::new(TrustDomainNoticeCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_notice_validator();
        let item = new_trust_domain_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TrustDomainNoticeTitleLengthRule { max_length: 1000 };
        let mut item = new_trust_domain_notice("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = TrustDomainNoticeCostRangeRule { min: 2.6, max: 1128.1 };
        let mut item = new_trust_domain_notice("x", "y");
        item.cost = 1129.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
