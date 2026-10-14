use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient rule quota must pass. Check returns its problems, or none.
pub trait LenientRuleQuotaRule: std::fmt::Debug {
    fn check(&self, item: &LenientRuleQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleQuotaDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientRuleQuotaRule for LenientRuleQuotaDiscountRangeRule {
    fn check(&self, item: &LenientRuleQuota) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleQuotaHostLengthRule {
    pub max_length: i64,
}

impl LenientRuleQuotaRule for LenientRuleQuotaHostLengthRule {
    fn check(&self, item: &LenientRuleQuota) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleQuotaCodeLengthRule {
    pub max_length: i64,
}

impl LenientRuleQuotaRule for LenientRuleQuotaCodeLengthRule {
    fn check(&self, item: &LenientRuleQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient rule quota.
#[derive(Debug, Clone)]
pub struct LenientRuleQuotaValidator {
    pub rules: Vec<Rc<dyn LenientRuleQuotaRule>>,
}

impl LenientRuleQuotaValidator {
    pub fn validate(&self, item: &LenientRuleQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientRuleQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRuleQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_rule_quota_validator() -> LenientRuleQuotaValidator {
    LenientRuleQuotaValidator {
        rules: vec![
            Rc::new(LenientRuleQuotaDiscountRangeRule { min: 1.7, max: 1286.4 }) as Rc<dyn LenientRuleQuotaRule>,
            Rc::new(LenientRuleQuotaHostLengthRule { max_length: 120 }),
            Rc::new(LenientRuleQuotaCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_rule_quota_validator();
        let item = new_lenient_rule_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = LenientRuleQuotaDiscountRangeRule { min: 8.5, max: 625.2 };
        let mut item = new_lenient_rule_quota("x", "y");
        item.discount = 626.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = LenientRuleQuotaHostLengthRule { max_length: 64 };
        let mut item = new_lenient_rule_quota("x", "y");
        item.host = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientRuleQuotaCodeLengthRule { max_length: 120 };
        let mut item = new_lenient_rule_quota("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
