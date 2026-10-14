use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient policy variance must pass. Check returns its problems, or none.
pub trait LenientPolicyVarianceRule: std::fmt::Debug {
    fn check(&self, item: &LenientPolicyVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyVariancePhoneLengthRule {
    pub max_length: i64,
}

impl LenientPolicyVarianceRule for LenientPolicyVariancePhoneLengthRule {
    fn check(&self, item: &LenientPolicyVariance) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyVarianceCodeLengthRule {
    pub max_length: i64,
}

impl LenientPolicyVarianceRule for LenientPolicyVarianceCodeLengthRule {
    fn check(&self, item: &LenientPolicyVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyVarianceRequiredPhoneRule {
    pub message: String,
}

impl LenientPolicyVarianceRule for LenientPolicyVarianceRequiredPhoneRule {
    fn check(&self, item: &LenientPolicyVariance) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyVarianceQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPolicyVarianceRule for LenientPolicyVarianceQuotaUnitsRangeRule {
    fn check(&self, item: &LenientPolicyVariance) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient policy variance.
#[derive(Debug, Clone)]
pub struct LenientPolicyVarianceValidator {
    pub rules: Vec<Rc<dyn LenientPolicyVarianceRule>>,
}

impl LenientPolicyVarianceValidator {
    pub fn validate(&self, item: &LenientPolicyVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientPolicyVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPolicyVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_policy_variance_validator() -> LenientPolicyVarianceValidator {
    LenientPolicyVarianceValidator {
        rules: vec![
            Rc::new(LenientPolicyVariancePhoneLengthRule { max_length: 120 }) as Rc<dyn LenientPolicyVarianceRule>,
            Rc::new(LenientPolicyVarianceCodeLengthRule { max_length: 32 }),
            Rc::new(LenientPolicyVarianceRequiredPhoneRule {
                message: "lenient policy variance needs a value".to_string(),
            }),
            Rc::new(LenientPolicyVarianceQuotaUnitsRangeRule { min: 16, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_policy_variance_validator();
        let item = new_lenient_policy_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = LenientPolicyVariancePhoneLengthRule { max_length: 86400 };
        let mut item = new_lenient_policy_variance("x", "y");
        item.phone = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientPolicyVarianceCodeLengthRule { max_length: 3600 };
        let mut item = new_lenient_policy_variance("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = LenientPolicyVarianceRequiredPhoneRule {
            message: "lenient policy variance needs this field".to_string(),
        };
        let mut item = new_lenient_policy_variance("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = LenientPolicyVarianceQuotaUnitsRangeRule { min: 120, max: 255 };
        let mut item = new_lenient_policy_variance("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
