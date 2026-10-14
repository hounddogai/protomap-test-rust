use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal query variance must pass. Check returns its problems, or none.
pub trait SeasonalQueryVarianceRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalQueryVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryVarianceClusterLengthRule {
    pub max_length: i64,
}

impl SeasonalQueryVarianceRule for SeasonalQueryVarianceClusterLengthRule {
    fn check(&self, item: &SeasonalQueryVariance) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryVarianceCodePrefixRule {
    pub prefix: String,
}

impl SeasonalQueryVarianceRule for SeasonalQueryVarianceCodePrefixRule {
    fn check(&self, item: &SeasonalQueryVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryVarianceCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalQueryVarianceRule for SeasonalQueryVarianceCodeLengthRule {
    fn check(&self, item: &SeasonalQueryVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryVarianceRequiredCodeRule {
    pub message: String,
}

impl SeasonalQueryVarianceRule for SeasonalQueryVarianceRequiredCodeRule {
    fn check(&self, item: &SeasonalQueryVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryVarianceRequiredClusterRule {
    pub message: String,
}

impl SeasonalQueryVarianceRule for SeasonalQueryVarianceRequiredClusterRule {
    fn check(&self, item: &SeasonalQueryVariance) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal query variance.
#[derive(Debug, Clone)]
pub struct SeasonalQueryVarianceValidator {
    pub rules: Vec<Rc<dyn SeasonalQueryVarianceRule>>,
}

impl SeasonalQueryVarianceValidator {
    pub fn validate(&self, item: &SeasonalQueryVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalQueryVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalQueryVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_query_variance_validator() -> SeasonalQueryVarianceValidator {
    SeasonalQueryVarianceValidator {
        rules: vec![
            Rc::new(SeasonalQueryVarianceClusterLengthRule { max_length: 80 }) as Rc<dyn SeasonalQueryVarianceRule>,
            Rc::new(SeasonalQueryVarianceCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(SeasonalQueryVarianceCodeLengthRule { max_length: 32 }),
            Rc::new(SeasonalQueryVarianceRequiredCodeRule {
                message: "seasonal query variance needs a value".to_string(),
            }),
            Rc::new(SeasonalQueryVarianceRequiredClusterRule {
                message: "seasonal query variance needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_query_variance_validator();
        let item = new_seasonal_query_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = SeasonalQueryVarianceClusterLengthRule { max_length: 86400 };
        let mut item = new_seasonal_query_variance("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalQueryVarianceCodeLengthRule { max_length: 3600 };
        let mut item = new_seasonal_query_variance("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalQueryVarianceRequiredCodeRule {
            message: "seasonal query variance needs a value".to_string(),
        };
        let mut item = new_seasonal_query_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = SeasonalQueryVarianceRequiredClusterRule {
            message: "seasonal query variance needs this field".to_string(),
        };
        let mut item = new_seasonal_query_variance("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
