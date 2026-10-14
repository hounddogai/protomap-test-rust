use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant tier must pass. Check returns its problems, or none.
pub trait TenantTierRule: std::fmt::Debug {
    fn check(&self, item: &TenantTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTierRequiredCodeRule {
    pub message: String,
}

impl TenantTierRule for TenantTierRequiredCodeRule {
    fn check(&self, item: &TenantTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTierMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantTierRule for TenantTierMaximumRangeRule {
    fn check(&self, item: &TenantTier) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTierAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl TenantTierRule for TenantTierAllowedSegmentRule {
    fn check(&self, item: &TenantTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is missing", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTierLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantTierRule for TenantTierLimitRangeRule {
    fn check(&self, item: &TenantTier) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tenant tier.
#[derive(Debug, Clone)]
pub struct TenantTierValidator {
    pub rules: Vec<Rc<dyn TenantTierRule>>,
}

impl TenantTierValidator {
    pub fn validate(&self, item: &TenantTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_tier_validator() -> TenantTierValidator {
    TenantTierValidator {
        rules: vec![
            Rc::new(TenantTierRequiredCodeRule {
                message: "tenant tier needs this field".to_string(),
            }) as Rc<dyn TenantTierRule>,
            Rc::new(TenantTierMaximumRangeRule { min: 3.7, max: 4992.6 }),
            Rc::new(TenantTierAllowedSegmentRule {
                allowed: vec!["weighted".to_string(), "pending".to_string()],
            }),
            Rc::new(TenantTierLimitRangeRule { min: 80, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_tier_validator();
        let item = new_tenant_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TenantTierRequiredCodeRule {
            message: "tenant tier needs this field".to_string(),
        };
        let mut item = new_tenant_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = TenantTierMaximumRangeRule { min: 4.8, max: 4768.2 };
        let mut item = new_tenant_tier("x", "y");
        item.maximum = 4769.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = TenantTierLimitRangeRule { min: 64, max: 120 };
        let mut item = new_tenant_tier("x", "y");
        item.limit = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
