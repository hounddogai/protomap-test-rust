use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled tenant agreement must pass. Check returns its problems, or none.
pub trait ScheduledTenantAgreementRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledTenantAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantAgreementLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledTenantAgreementRule for ScheduledTenantAgreementLabelLengthRule {
    fn check(&self, item: &ScheduledTenantAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantAgreementPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledTenantAgreementRule for ScheduledTenantAgreementPortRangeRule {
    fn check(&self, item: &ScheduledTenantAgreement) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantAgreementRequiredCodeRule {
    pub message: String,
}

impl ScheduledTenantAgreementRule for ScheduledTenantAgreementRequiredCodeRule {
    fn check(&self, item: &ScheduledTenantAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantAgreementAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ScheduledTenantAgreementRule for ScheduledTenantAgreementAllowedLabelRule {
    fn check(&self, item: &ScheduledTenantAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantAgreementScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledTenantAgreementRule for ScheduledTenantAgreementScoreRangeRule {
    fn check(&self, item: &ScheduledTenantAgreement) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantAgreementRequiredLocaleRule {
    pub message: String,
}

impl ScheduledTenantAgreementRule for ScheduledTenantAgreementRequiredLocaleRule {
    fn check(&self, item: &ScheduledTenantAgreement) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled tenant agreement.
#[derive(Debug, Clone)]
pub struct ScheduledTenantAgreementValidator {
    pub rules: Vec<Rc<dyn ScheduledTenantAgreementRule>>,
}

impl ScheduledTenantAgreementValidator {
    pub fn validate(&self, item: &ScheduledTenantAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledTenantAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledTenantAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_tenant_agreement_validator() -> ScheduledTenantAgreementValidator {
    ScheduledTenantAgreementValidator {
        rules: vec![
            Rc::new(ScheduledTenantAgreementLabelLengthRule { max_length: 80 }) as Rc<dyn ScheduledTenantAgreementRule>,
            Rc::new(ScheduledTenantAgreementPortRangeRule { min: 120, max: 86400 }),
            Rc::new(ScheduledTenantAgreementRequiredCodeRule {
                message: "scheduled tenant agreement needs an entry".to_string(),
            }),
            Rc::new(ScheduledTenantAgreementAllowedLabelRule {
                allowed: vec!["rolling".to_string(), "legacy".to_string(), "lenient".to_string()],
            }),
            Rc::new(ScheduledTenantAgreementScoreRangeRule { min: 6.6, max: 1138.5 }),
            Rc::new(ScheduledTenantAgreementRequiredLocaleRule {
                message: "scheduled tenant agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_tenant_agreement_validator();
        let item = new_scheduled_tenant_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledTenantAgreementLabelLengthRule { max_length: 64 };
        let mut item = new_scheduled_tenant_agreement("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = ScheduledTenantAgreementPortRangeRule { min: 16, max: 255 };
        let mut item = new_scheduled_tenant_agreement("x", "y");
        item.port = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledTenantAgreementRequiredCodeRule {
            message: "scheduled tenant agreement needs an entry".to_string(),
        };
        let mut item = new_scheduled_tenant_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ScheduledTenantAgreementScoreRangeRule { min: 4.8, max: 1366.8 };
        let mut item = new_scheduled_tenant_agreement("x", "y");
        item.score = 1368.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = ScheduledTenantAgreementRequiredLocaleRule {
            message: "scheduled tenant agreement needs an entry".to_string(),
        };
        let mut item = new_scheduled_tenant_agreement("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
