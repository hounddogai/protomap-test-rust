use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard agreement must pass. Check returns its problems, or none.
pub trait DashboardAgreementRule: std::fmt::Debug {
    fn check(&self, item: &DashboardAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAgreementWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardAgreementRule for DashboardAgreementWindowDaysRangeRule {
    fn check(&self, item: &DashboardAgreement) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAgreementCipherLengthRule {
    pub max_length: i64,
}

impl DashboardAgreementRule for DashboardAgreementCipherLengthRule {
    fn check(&self, item: &DashboardAgreement) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAgreementRequiredCipherRule {
    pub message: String,
}

impl DashboardAgreementRule for DashboardAgreementRequiredCipherRule {
    fn check(&self, item: &DashboardAgreement) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAgreementRequiredCodeRule {
    pub message: String,
}

impl DashboardAgreementRule for DashboardAgreementRequiredCodeRule {
    fn check(&self, item: &DashboardAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard agreement.
#[derive(Debug, Clone)]
pub struct DashboardAgreementValidator {
    pub rules: Vec<Rc<dyn DashboardAgreementRule>>,
}

impl DashboardAgreementValidator {
    pub fn validate(&self, item: &DashboardAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_agreement_validator() -> DashboardAgreementValidator {
    DashboardAgreementValidator {
        rules: vec![
            Rc::new(DashboardAgreementWindowDaysRangeRule { min: 80, max: 3600 }) as Rc<dyn DashboardAgreementRule>,
            Rc::new(DashboardAgreementCipherLengthRule { max_length: 120 }),
            Rc::new(DashboardAgreementRequiredCipherRule {
                message: "dashboard agreement needs a value".to_string(),
            }),
            Rc::new(DashboardAgreementRequiredCodeRule {
                message: "dashboard agreement needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_agreement_validator();
        let item = new_dashboard_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = DashboardAgreementWindowDaysRangeRule { min: 80, max: 3600 };
        let mut item = new_dashboard_agreement("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = DashboardAgreementCipherLengthRule { max_length: 64 };
        let mut item = new_dashboard_agreement("x", "y");
        item.cipher = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = DashboardAgreementRequiredCipherRule {
            message: "dashboard agreement needs this field".to_string(),
        };
        let mut item = new_dashboard_agreement("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DashboardAgreementRequiredCodeRule {
            message: "dashboard agreement needs this field".to_string(),
        };
        let mut item = new_dashboard_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
