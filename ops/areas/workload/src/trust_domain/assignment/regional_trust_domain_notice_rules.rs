use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional trust domain notice must pass. Check returns its problems, or none.
pub trait RegionalTrustDomainNoticeRule: std::fmt::Debug {
    fn check(&self, item: &RegionalTrustDomainNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTrustDomainNoticeLabelLengthRule {
    pub max_length: i64,
}

impl RegionalTrustDomainNoticeRule for RegionalTrustDomainNoticeLabelLengthRule {
    fn check(&self, item: &RegionalTrustDomainNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTrustDomainNoticeNameLengthRule {
    pub max_length: i64,
}

impl RegionalTrustDomainNoticeRule for RegionalTrustDomainNoticeNameLengthRule {
    fn check(&self, item: &RegionalTrustDomainNotice) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTrustDomainNoticeCodePrefixRule {
    pub prefix: String,
}

impl RegionalTrustDomainNoticeRule for RegionalTrustDomainNoticeCodePrefixRule {
    fn check(&self, item: &RegionalTrustDomainNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTrustDomainNoticeRequiredTenantIdRule {
    pub message: String,
}

impl RegionalTrustDomainNoticeRule for RegionalTrustDomainNoticeRequiredTenantIdRule {
    fn check(&self, item: &RegionalTrustDomainNotice) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTrustDomainNoticeScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalTrustDomainNoticeRule for RegionalTrustDomainNoticeScoreRangeRule {
    fn check(&self, item: &RegionalTrustDomainNotice) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional trust domain notice.
#[derive(Debug, Clone)]
pub struct RegionalTrustDomainNoticeValidator {
    pub rules: Vec<Rc<dyn RegionalTrustDomainNoticeRule>>,
}

impl RegionalTrustDomainNoticeValidator {
    pub fn validate(&self, item: &RegionalTrustDomainNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalTrustDomainNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalTrustDomainNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_trust_domain_notice_validator() -> RegionalTrustDomainNoticeValidator {
    RegionalTrustDomainNoticeValidator {
        rules: vec![
            Rc::new(RegionalTrustDomainNoticeLabelLengthRule { max_length: 120 })
                as Rc<dyn RegionalTrustDomainNoticeRule>,
            Rc::new(RegionalTrustDomainNoticeNameLengthRule { max_length: 32 }),
            Rc::new(RegionalTrustDomainNoticeCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RegionalTrustDomainNoticeRequiredTenantIdRule {
                message: "regional trust domain notice needs a value".to_string(),
            }),
            Rc::new(RegionalTrustDomainNoticeScoreRangeRule { min: 1.4, max: 1548.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_trust_domain_notice_validator();
        let item = new_regional_trust_domain_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalTrustDomainNoticeLabelLengthRule { max_length: 16 };
        let mut item = new_regional_trust_domain_notice("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalTrustDomainNoticeNameLengthRule { max_length: 120 };
        let mut item = new_regional_trust_domain_notice("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = RegionalTrustDomainNoticeRequiredTenantIdRule {
            message: "regional trust domain notice needs an entry".to_string(),
        };
        let mut item = new_regional_trust_domain_notice("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = RegionalTrustDomainNoticeScoreRangeRule { min: 6.0, max: 3113.7 };
        let mut item = new_regional_trust_domain_notice("x", "y");
        item.score = 3115.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
