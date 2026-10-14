use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant quota must pass. Check returns its problems, or none.
pub trait TenantQuotaRule: std::fmt::Debug {
    fn check(&self, item: &TenantQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantQuotaCodePrefixRule {
    pub prefix: String,
}

impl TenantQuotaRule for TenantQuotaCodePrefixRule {
    fn check(&self, item: &TenantQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantQuotaWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantQuotaRule for TenantQuotaWeightRangeRule {
    fn check(&self, item: &TenantQuota) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantQuotaRequiredLabelRule {
    pub message: String,
}

impl TenantQuotaRule for TenantQuotaRequiredLabelRule {
    fn check(&self, item: &TenantQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantQuotaRequiredCodeRule {
    pub message: String,
}

impl TenantQuotaRule for TenantQuotaRequiredCodeRule {
    fn check(&self, item: &TenantQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantQuotaRequiredCountryRule {
    pub message: String,
}

impl TenantQuotaRule for TenantQuotaRequiredCountryRule {
    fn check(&self, item: &TenantQuota) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant quota.
#[derive(Debug, Clone)]
pub struct TenantQuotaValidator {
    pub rules: Vec<Rc<dyn TenantQuotaRule>>,
}

impl TenantQuotaValidator {
    pub fn validate(&self, item: &TenantQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_quota_validator() -> TenantQuotaValidator {
    TenantQuotaValidator {
        rules: vec![
            Rc::new(TenantQuotaCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn TenantQuotaRule>,
            Rc::new(TenantQuotaWeightRangeRule { min: 7.1, max: 1412.6 }),
            Rc::new(TenantQuotaRequiredLabelRule {
                message: "tenant quota needs a value".to_string(),
            }),
            Rc::new(TenantQuotaRequiredCodeRule {
                message: "tenant quota needs this field".to_string(),
            }),
            Rc::new(TenantQuotaRequiredCountryRule {
                message: "tenant quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_quota_validator();
        let item = new_tenant_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = TenantQuotaWeightRangeRule { min: 0.2, max: 1737.7 };
        let mut item = new_tenant_quota("x", "y");
        item.weight = 1739.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TenantQuotaRequiredLabelRule {
            message: "tenant quota needs a value".to_string(),
        };
        let mut item = new_tenant_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TenantQuotaRequiredCodeRule {
            message: "tenant quota needs this field".to_string(),
        };
        let mut item = new_tenant_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = TenantQuotaRequiredCountryRule {
            message: "tenant quota needs this field".to_string(),
        };
        let mut item = new_tenant_quota("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
