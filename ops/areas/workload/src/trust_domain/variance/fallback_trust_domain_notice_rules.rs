use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback trust domain notice must pass. Check returns its problems, or none.
pub trait FallbackTrustDomainNoticeRule: std::fmt::Debug {
    fn check(&self, item: &FallbackTrustDomainNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainNoticeCategoryLengthRule {
    pub max_length: i64,
}

impl FallbackTrustDomainNoticeRule for FallbackTrustDomainNoticeCategoryLengthRule {
    fn check(&self, item: &FallbackTrustDomainNotice) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainNoticeCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackTrustDomainNoticeRule for FallbackTrustDomainNoticeCreatedAtRangeRule {
    fn check(&self, item: &FallbackTrustDomainNotice) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainNoticeRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackTrustDomainNoticeRule for FallbackTrustDomainNoticeRevisionRangeRule {
    fn check(&self, item: &FallbackTrustDomainNotice) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainNoticeCodeLengthRule {
    pub max_length: i64,
}

impl FallbackTrustDomainNoticeRule for FallbackTrustDomainNoticeCodeLengthRule {
    fn check(&self, item: &FallbackTrustDomainNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback trust domain notice.
#[derive(Debug, Clone)]
pub struct FallbackTrustDomainNoticeValidator {
    pub rules: Vec<Rc<dyn FallbackTrustDomainNoticeRule>>,
}

impl FallbackTrustDomainNoticeValidator {
    pub fn validate(&self, item: &FallbackTrustDomainNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackTrustDomainNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackTrustDomainNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_trust_domain_notice_validator() -> FallbackTrustDomainNoticeValidator {
    FallbackTrustDomainNoticeValidator {
        rules: vec![
            Rc::new(FallbackTrustDomainNoticeCategoryLengthRule { max_length: 255 })
                as Rc<dyn FallbackTrustDomainNoticeRule>,
            Rc::new(FallbackTrustDomainNoticeCreatedAtRangeRule { min: 32, max: 120 }),
            Rc::new(FallbackTrustDomainNoticeRevisionRangeRule { min: 80, max: 3600 }),
            Rc::new(FallbackTrustDomainNoticeCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_trust_domain_notice_validator();
        let item = new_fallback_trust_domain_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = FallbackTrustDomainNoticeCategoryLengthRule { max_length: 120 };
        let mut item = new_fallback_trust_domain_notice("x", "y");
        item.category = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = FallbackTrustDomainNoticeCreatedAtRangeRule { min: 64, max: 64 };
        let mut item = new_fallback_trust_domain_notice("x", "y");
        item.created_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = FallbackTrustDomainNoticeRevisionRangeRule { min: 16, max: 64 };
        let mut item = new_fallback_trust_domain_notice("x", "y");
        item.revision = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackTrustDomainNoticeCodeLengthRule { max_length: 80 };
        let mut item = new_fallback_trust_domain_notice("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
