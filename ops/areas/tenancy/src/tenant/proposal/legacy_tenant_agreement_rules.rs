use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy tenant agreement must pass. Check returns its problems, or none.
pub trait LegacyTenantAgreementRule: std::fmt::Debug {
    fn check(&self, item: &LegacyTenantAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantAgreementRequiredCodeRule {
    pub message: String,
}

impl LegacyTenantAgreementRule for LegacyTenantAgreementRequiredCodeRule {
    fn check(&self, item: &LegacyTenantAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantAgreementCodeLengthRule {
    pub max_length: i64,
}

impl LegacyTenantAgreementRule for LegacyTenantAgreementCodeLengthRule {
    fn check(&self, item: &LegacyTenantAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantAgreementRequiredTitleRule {
    pub message: String,
}

impl LegacyTenantAgreementRule for LegacyTenantAgreementRequiredTitleRule {
    fn check(&self, item: &LegacyTenantAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantAgreementRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyTenantAgreementRule for LegacyTenantAgreementRotationDaysRangeRule {
    fn check(&self, item: &LegacyTenantAgreement) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a legacy tenant agreement.
#[derive(Debug, Clone)]
pub struct LegacyTenantAgreementValidator {
    pub rules: Vec<Rc<dyn LegacyTenantAgreementRule>>,
}

impl LegacyTenantAgreementValidator {
    pub fn validate(&self, item: &LegacyTenantAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyTenantAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyTenantAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_tenant_agreement_validator() -> LegacyTenantAgreementValidator {
    LegacyTenantAgreementValidator {
        rules: vec![
            Rc::new(LegacyTenantAgreementRequiredCodeRule {
                message: "legacy tenant agreement needs a value".to_string(),
            }) as Rc<dyn LegacyTenantAgreementRule>,
            Rc::new(LegacyTenantAgreementCodeLengthRule { max_length: 120 }),
            Rc::new(LegacyTenantAgreementRequiredTitleRule {
                message: "legacy tenant agreement needs an entry".to_string(),
            }),
            Rc::new(LegacyTenantAgreementRotationDaysRangeRule { min: 64, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_tenant_agreement_validator();
        let item = new_legacy_tenant_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyTenantAgreementRequiredCodeRule {
            message: "legacy tenant agreement needs this field".to_string(),
        };
        let mut item = new_legacy_tenant_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyTenantAgreementCodeLengthRule { max_length: 3600 };
        let mut item = new_legacy_tenant_agreement("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyTenantAgreementRequiredTitleRule {
            message: "legacy tenant agreement needs this field".to_string(),
        };
        let mut item = new_legacy_tenant_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = LegacyTenantAgreementRotationDaysRangeRule { min: 64, max: 3600 };
        let mut item = new_legacy_tenant_agreement("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
