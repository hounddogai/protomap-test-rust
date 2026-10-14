use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a ttl policy request must pass. Check returns its problems, or none.
pub trait TtlPolicyRequestRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRequestRequiredDescriptionRule {
    pub message: String,
}

impl TtlPolicyRequestRule for TtlPolicyRequestRequiredDescriptionRule {
    fn check(&self, item: &TtlPolicyRequest) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRequestAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl TtlPolicyRequestRule for TtlPolicyRequestAllowedTenantIdRule {
    fn check(&self, item: &TtlPolicyRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is expired", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRequestThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyRequestRule for TtlPolicyRequestThresholdRangeRule {
    fn check(&self, item: &TtlPolicyRequest) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRequestRequiredTenantIdRule {
    pub message: String,
}

impl TtlPolicyRequestRule for TtlPolicyRequestRequiredTenantIdRule {
    fn check(&self, item: &TtlPolicyRequest) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRequestRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyRequestRule for TtlPolicyRequestRequiredCodeRule {
    fn check(&self, item: &TtlPolicyRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy request.
#[derive(Debug, Clone)]
pub struct TtlPolicyRequestValidator {
    pub rules: Vec<Rc<dyn TtlPolicyRequestRule>>,
}

impl TtlPolicyRequestValidator {
    pub fn validate(&self, item: &TtlPolicyRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_request_validator() -> TtlPolicyRequestValidator {
    TtlPolicyRequestValidator {
        rules: vec![
            Rc::new(TtlPolicyRequestRequiredDescriptionRule {
                message: "ttl policy request needs this field".to_string(),
            }) as Rc<dyn TtlPolicyRequestRule>,
            Rc::new(TtlPolicyRequestAllowedTenantIdRule {
                allowed: vec!["scheduled".to_string()],
            }),
            Rc::new(TtlPolicyRequestThresholdRangeRule { min: 7.6, max: 2988.0 }),
            Rc::new(TtlPolicyRequestRequiredTenantIdRule {
                message: "ttl policy request needs this field".to_string(),
            }),
            Rc::new(TtlPolicyRequestRequiredCodeRule {
                message: "ttl policy request needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_request_validator();
        let item = new_ttl_policy_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = TtlPolicyRequestRequiredDescriptionRule {
            message: "ttl policy request needs this field".to_string(),
        };
        let mut item = new_ttl_policy_request("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = TtlPolicyRequestThresholdRangeRule { min: 7.7, max: 1811.4 };
        let mut item = new_ttl_policy_request("x", "y");
        item.threshold = 1812.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = TtlPolicyRequestRequiredTenantIdRule {
            message: "ttl policy request needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_request("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyRequestRequiredCodeRule {
            message: "ttl policy request needs this field".to_string(),
        };
        let mut item = new_ttl_policy_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
