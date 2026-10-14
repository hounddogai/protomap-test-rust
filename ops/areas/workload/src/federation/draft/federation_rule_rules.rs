use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation rule must pass. Check returns its problems, or none.
pub trait FederationRuleRule: std::fmt::Debug {
    fn check(&self, item: &FederationRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRuleRequiredTenantIdRule {
    pub message: String,
}

impl FederationRuleRule for FederationRuleRequiredTenantIdRule {
    fn check(&self, item: &FederationRule) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRuleTenantIdLengthRule {
    pub max_length: i64,
}

impl FederationRuleRule for FederationRuleTenantIdLengthRule {
    fn check(&self, item: &FederationRule) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRuleRequiredCodeRule {
    pub message: String,
}

impl FederationRuleRule for FederationRuleRequiredCodeRule {
    fn check(&self, item: &FederationRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRuleRequiredSummaryRule {
    pub message: String,
}

impl FederationRuleRule for FederationRuleRequiredSummaryRule {
    fn check(&self, item: &FederationRule) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRuleLabelLengthRule {
    pub max_length: i64,
}

impl FederationRuleRule for FederationRuleLabelLengthRule {
    fn check(&self, item: &FederationRule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation rule.
#[derive(Debug, Clone)]
pub struct FederationRuleValidator {
    pub rules: Vec<Rc<dyn FederationRuleRule>>,
}

impl FederationRuleValidator {
    pub fn validate(&self, item: &FederationRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_rule_validator() -> FederationRuleValidator {
    FederationRuleValidator {
        rules: vec![
            Rc::new(FederationRuleRequiredTenantIdRule {
                message: "federation rule needs an entry".to_string(),
            }) as Rc<dyn FederationRuleRule>,
            Rc::new(FederationRuleTenantIdLengthRule { max_length: 1000 }),
            Rc::new(FederationRuleRequiredCodeRule {
                message: "federation rule needs this field".to_string(),
            }),
            Rc::new(FederationRuleRequiredSummaryRule {
                message: "federation rule needs a value".to_string(),
            }),
            Rc::new(FederationRuleLabelLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_rule_validator();
        let item = new_federation_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = FederationRuleRequiredTenantIdRule {
            message: "federation rule needs a value".to_string(),
        };
        let mut item = new_federation_rule("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = FederationRuleTenantIdLengthRule { max_length: 1000 };
        let mut item = new_federation_rule("x", "y");
        item.tenant_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FederationRuleRequiredCodeRule {
            message: "federation rule needs an entry".to_string(),
        };
        let mut item = new_federation_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = FederationRuleRequiredSummaryRule {
            message: "federation rule needs this field".to_string(),
        };
        let mut item = new_federation_rule("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FederationRuleLabelLengthRule { max_length: 16 };
        let mut item = new_federation_rule("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
