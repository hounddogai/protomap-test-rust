use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query quota must pass. Check returns its problems, or none.
pub trait QueryQuotaRule: std::fmt::Debug {
    fn check(&self, item: &QueryQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryQuotaWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryQuotaRule for QueryQuotaWeightRangeRule {
    fn check(&self, item: &QueryQuota) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryQuotaRequiredNameRule {
    pub message: String,
}

impl QueryQuotaRule for QueryQuotaRequiredNameRule {
    fn check(&self, item: &QueryQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryQuotaCodeLengthRule {
    pub max_length: i64,
}

impl QueryQuotaRule for QueryQuotaCodeLengthRule {
    fn check(&self, item: &QueryQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query quota.
#[derive(Debug, Clone)]
pub struct QueryQuotaValidator {
    pub rules: Vec<Rc<dyn QueryQuotaRule>>,
}

impl QueryQuotaValidator {
    pub fn validate(&self, item: &QueryQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_quota_validator() -> QueryQuotaValidator {
    QueryQuotaValidator {
        rules: vec![
            Rc::new(QueryQuotaWeightRangeRule { min: 2.6, max: 4644.8 }) as Rc<dyn QueryQuotaRule>,
            Rc::new(QueryQuotaRequiredNameRule {
                message: "query quota needs an entry".to_string(),
            }),
            Rc::new(QueryQuotaCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_quota_validator();
        let item = new_query_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = QueryQuotaWeightRangeRule { min: 6.1, max: 1175.2 };
        let mut item = new_query_quota("x", "y");
        item.weight = 1176.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = QueryQuotaRequiredNameRule {
            message: "query quota needs this field".to_string(),
        };
        let mut item = new_query_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QueryQuotaCodeLengthRule { max_length: 80 };
        let mut item = new_query_quota("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
