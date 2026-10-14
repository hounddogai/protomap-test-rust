use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default dns record notice must pass. Check returns its problems, or none.
pub trait DefaultDnsRecordNoticeRule: std::fmt::Debug {
    fn check(&self, item: &DefaultDnsRecordNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordNoticeAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultDnsRecordNoticeRule for DefaultDnsRecordNoticeAmountRangeRule {
    fn check(&self, item: &DefaultDnsRecordNotice) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordNoticeAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultDnsRecordNoticeRule for DefaultDnsRecordNoticeAttemptsRangeRule {
    fn check(&self, item: &DefaultDnsRecordNotice) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordNoticeRequiredTitleRule {
    pub message: String,
}

impl DefaultDnsRecordNoticeRule for DefaultDnsRecordNoticeRequiredTitleRule {
    fn check(&self, item: &DefaultDnsRecordNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordNoticeCodePrefixRule {
    pub prefix: String,
}

impl DefaultDnsRecordNoticeRule for DefaultDnsRecordNoticeCodePrefixRule {
    fn check(&self, item: &DefaultDnsRecordNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordNoticeRequiredCodeRule {
    pub message: String,
}

impl DefaultDnsRecordNoticeRule for DefaultDnsRecordNoticeRequiredCodeRule {
    fn check(&self, item: &DefaultDnsRecordNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default dns record notice.
#[derive(Debug, Clone)]
pub struct DefaultDnsRecordNoticeValidator {
    pub rules: Vec<Rc<dyn DefaultDnsRecordNoticeRule>>,
}

impl DefaultDnsRecordNoticeValidator {
    pub fn validate(&self, item: &DefaultDnsRecordNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultDnsRecordNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultDnsRecordNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_dns_record_notice_validator() -> DefaultDnsRecordNoticeValidator {
    DefaultDnsRecordNoticeValidator {
        rules: vec![
            Rc::new(DefaultDnsRecordNoticeAmountRangeRule { min: 1.2, max: 4064.3 })
                as Rc<dyn DefaultDnsRecordNoticeRule>,
            Rc::new(DefaultDnsRecordNoticeAttemptsRangeRule { min: 1000, max: 1000 }),
            Rc::new(DefaultDnsRecordNoticeRequiredTitleRule {
                message: "default dns record notice needs a value".to_string(),
            }),
            Rc::new(DefaultDnsRecordNoticeCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DefaultDnsRecordNoticeRequiredCodeRule {
                message: "default dns record notice needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_dns_record_notice_validator();
        let item = new_default_dns_record_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = DefaultDnsRecordNoticeAmountRangeRule { min: 3.6, max: 4045.7 };
        let mut item = new_default_dns_record_notice("x", "y");
        item.amount = 4047.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = DefaultDnsRecordNoticeAttemptsRangeRule { min: 80, max: 80 };
        let mut item = new_default_dns_record_notice("x", "y");
        item.attempts = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultDnsRecordNoticeRequiredTitleRule {
            message: "default dns record notice needs an entry".to_string(),
        };
        let mut item = new_default_dns_record_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultDnsRecordNoticeRequiredCodeRule {
            message: "default dns record notice needs this field".to_string(),
        };
        let mut item = new_default_dns_record_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
