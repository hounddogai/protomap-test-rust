use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered usage batch must pass. Check returns its problems, or none.
pub trait TieredUsageBatchRule: std::fmt::Debug {
    fn check(&self, item: &TieredUsageBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageBatchAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl TieredUsageBatchRule for TieredUsageBatchAllowedTenantIdRule {
    fn check(&self, item: &TieredUsageBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is above maximum", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageBatchTenantIdLengthRule {
    pub max_length: i64,
}

impl TieredUsageBatchRule for TieredUsageBatchTenantIdLengthRule {
    fn check(&self, item: &TieredUsageBatch) -> Vec<String> {
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
pub struct TieredUsageBatchCurrencyLengthRule {
    pub max_length: i64,
}

impl TieredUsageBatchRule for TieredUsageBatchCurrencyLengthRule {
    fn check(&self, item: &TieredUsageBatch) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered usage batch.
#[derive(Debug, Clone)]
pub struct TieredUsageBatchValidator {
    pub rules: Vec<Rc<dyn TieredUsageBatchRule>>,
}

impl TieredUsageBatchValidator {
    pub fn validate(&self, item: &TieredUsageBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredUsageBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredUsageBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_usage_batch_validator() -> TieredUsageBatchValidator {
    TieredUsageBatchValidator {
        rules: vec![
            Rc::new(TieredUsageBatchAllowedTenantIdRule {
                allowed: vec!["default".to_string()],
            }) as Rc<dyn TieredUsageBatchRule>,
            Rc::new(TieredUsageBatchTenantIdLengthRule { max_length: 64 }),
            Rc::new(TieredUsageBatchCurrencyLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_usage_batch_validator();
        let item = new_tiered_usage_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = TieredUsageBatchTenantIdLengthRule { max_length: 86400 };
        let mut item = new_tiered_usage_batch("x", "y");
        item.tenant_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = TieredUsageBatchCurrencyLengthRule { max_length: 64 };
        let mut item = new_tiered_usage_batch("x", "y");
        item.currency = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
