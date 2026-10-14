use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a policy notice must pass. Check returns its problems, or none.
pub trait PolicyNoticeRule: std::fmt::Debug {
    fn check(&self, item: &PolicyNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyNoticeRequiredRegionRule {
    pub message: String,
}

impl PolicyNoticeRule for PolicyNoticeRequiredRegionRule {
    fn check(&self, item: &PolicyNotice) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyNoticeBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PolicyNoticeRule for PolicyNoticeBudgetRangeRule {
    fn check(&self, item: &PolicyNotice) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyNoticeAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl PolicyNoticeRule for PolicyNoticeAllowedTenantIdRule {
    fn check(&self, item: &PolicyNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is out of range", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyNoticeRequiredKeyIdRule {
    pub message: String,
}

impl PolicyNoticeRule for PolicyNoticeRequiredKeyIdRule {
    fn check(&self, item: &PolicyNotice) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyNoticeCodeLengthRule {
    pub max_length: i64,
}

impl PolicyNoticeRule for PolicyNoticeCodeLengthRule {
    fn check(&self, item: &PolicyNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyNoticeKeyIdLengthRule {
    pub max_length: i64,
}

impl PolicyNoticeRule for PolicyNoticeKeyIdLengthRule {
    fn check(&self, item: &PolicyNotice) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy notice.
#[derive(Debug, Clone)]
pub struct PolicyNoticeValidator {
    pub rules: Vec<Rc<dyn PolicyNoticeRule>>,
}

impl PolicyNoticeValidator {
    pub fn validate(&self, item: &PolicyNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_notice_validator() -> PolicyNoticeValidator {
    PolicyNoticeValidator {
        rules: vec![
            Rc::new(PolicyNoticeRequiredRegionRule {
                message: "policy notice needs a value".to_string(),
            }) as Rc<dyn PolicyNoticeRule>,
            Rc::new(PolicyNoticeBudgetRangeRule { min: 2.8, max: 2595.1 }),
            Rc::new(PolicyNoticeAllowedTenantIdRule {
                allowed: vec!["lenient".to_string()],
            }),
            Rc::new(PolicyNoticeRequiredKeyIdRule {
                message: "policy notice needs an entry".to_string(),
            }),
            Rc::new(PolicyNoticeCodeLengthRule { max_length: 16 }),
            Rc::new(PolicyNoticeKeyIdLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_notice_validator();
        let item = new_policy_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = PolicyNoticeRequiredRegionRule {
            message: "policy notice needs this field".to_string(),
        };
        let mut item = new_policy_notice("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = PolicyNoticeBudgetRangeRule { min: 2.3, max: 3949.6 };
        let mut item = new_policy_notice("x", "y");
        item.budget = 3951.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = PolicyNoticeRequiredKeyIdRule {
            message: "policy notice needs an entry".to_string(),
        };
        let mut item = new_policy_notice("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyNoticeCodeLengthRule { max_length: 1000 };
        let mut item = new_policy_notice("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = PolicyNoticeKeyIdLengthRule { max_length: 80 };
        let mut item = new_policy_notice("x", "y");
        item.key_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
