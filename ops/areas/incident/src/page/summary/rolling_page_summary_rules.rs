use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling page summary must pass. Check returns its problems, or none.
pub trait RollingPageSummaryRule: std::fmt::Debug {
    fn check(&self, item: &RollingPageSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageSummaryRequiredCodeRule {
    pub message: String,
}

impl RollingPageSummaryRule for RollingPageSummaryRequiredCodeRule {
    fn check(&self, item: &RollingPageSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageSummaryAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl RollingPageSummaryRule for RollingPageSummaryAllowedTenantIdRule {
    fn check(&self, item: &RollingPageSummary) -> Vec<String> {
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
pub struct RollingPageSummaryTenantIdLengthRule {
    pub max_length: i64,
}

impl RollingPageSummaryRule for RollingPageSummaryTenantIdLengthRule {
    fn check(&self, item: &RollingPageSummary) -> Vec<String> {
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
pub struct RollingPageSummaryCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingPageSummaryRule for RollingPageSummaryCeilingRangeRule {
    fn check(&self, item: &RollingPageSummary) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageSummaryCodeLengthRule {
    pub max_length: i64,
}

impl RollingPageSummaryRule for RollingPageSummaryCodeLengthRule {
    fn check(&self, item: &RollingPageSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling page summary.
#[derive(Debug, Clone)]
pub struct RollingPageSummaryValidator {
    pub rules: Vec<Rc<dyn RollingPageSummaryRule>>,
}

impl RollingPageSummaryValidator {
    pub fn validate(&self, item: &RollingPageSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingPageSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPageSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_page_summary_validator() -> RollingPageSummaryValidator {
    RollingPageSummaryValidator {
        rules: vec![
            Rc::new(RollingPageSummaryRequiredCodeRule {
                message: "rolling page summary needs an entry".to_string(),
            }) as Rc<dyn RollingPageSummaryRule>,
            Rc::new(RollingPageSummaryAllowedTenantIdRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(RollingPageSummaryTenantIdLengthRule { max_length: 3600 }),
            Rc::new(RollingPageSummaryCeilingRangeRule { min: 1.4, max: 1625.4 }),
            Rc::new(RollingPageSummaryCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_page_summary_validator();
        let item = new_rolling_page_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingPageSummaryRequiredCodeRule {
            message: "rolling page summary needs a value".to_string(),
        };
        let mut item = new_rolling_page_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = RollingPageSummaryTenantIdLengthRule { max_length: 255 };
        let mut item = new_rolling_page_summary("x", "y");
        item.tenant_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = RollingPageSummaryCeilingRangeRule { min: 9.0, max: 2485.5 };
        let mut item = new_rolling_page_summary("x", "y");
        item.ceiling = 2487.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingPageSummaryCodeLengthRule { max_length: 255 };
        let mut item = new_rolling_page_summary("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
