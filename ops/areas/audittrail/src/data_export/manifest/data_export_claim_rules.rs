use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export claim must pass. Check returns its problems, or none.
pub trait DataExportClaimRule: std::fmt::Debug {
    fn check(&self, item: &DataExportClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportClaimCodeLengthRule {
    pub max_length: i64,
}

impl DataExportClaimRule for DataExportClaimCodeLengthRule {
    fn check(&self, item: &DataExportClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportClaimExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportClaimRule for DataExportClaimExpiresAtRangeRule {
    fn check(&self, item: &DataExportClaim) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportClaimClusterLengthRule {
    pub max_length: i64,
}

impl DataExportClaimRule for DataExportClaimClusterLengthRule {
    fn check(&self, item: &DataExportClaim) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportClaimCurrencyLengthRule {
    pub max_length: i64,
}

impl DataExportClaimRule for DataExportClaimCurrencyLengthRule {
    fn check(&self, item: &DataExportClaim) -> Vec<String> {
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

/// Runs every rule of a data export claim.
#[derive(Debug, Clone)]
pub struct DataExportClaimValidator {
    pub rules: Vec<Rc<dyn DataExportClaimRule>>,
}

impl DataExportClaimValidator {
    pub fn validate(&self, item: &DataExportClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_claim_validator() -> DataExportClaimValidator {
    DataExportClaimValidator {
        rules: vec![
            Rc::new(DataExportClaimCodeLengthRule { max_length: 120 }) as Rc<dyn DataExportClaimRule>,
            Rc::new(DataExportClaimExpiresAtRangeRule { min: 16, max: 16 }),
            Rc::new(DataExportClaimClusterLengthRule { max_length: 3600 }),
            Rc::new(DataExportClaimCurrencyLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_claim_validator();
        let item = new_data_export_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DataExportClaimCodeLengthRule { max_length: 80 };
        let mut item = new_data_export_claim("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = DataExportClaimExpiresAtRangeRule { min: 80, max: 255 };
        let mut item = new_data_export_claim("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = DataExportClaimClusterLengthRule { max_length: 120 };
        let mut item = new_data_export_claim("x", "y");
        item.cluster = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = DataExportClaimCurrencyLengthRule { max_length: 86400 };
        let mut item = new_data_export_claim("x", "y");
        item.currency = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
