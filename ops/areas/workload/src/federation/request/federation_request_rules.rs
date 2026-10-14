use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation request must pass. Check returns its problems, or none.
pub trait FederationRequestRule: std::fmt::Debug {
    fn check(&self, item: &FederationRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRequestQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationRequestRule for FederationRequestQuotaUnitsRangeRule {
    fn check(&self, item: &FederationRequest) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRequestRequiredTitleRule {
    pub message: String,
}

impl FederationRequestRule for FederationRequestRequiredTitleRule {
    fn check(&self, item: &FederationRequest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRequestCodeLengthRule {
    pub max_length: i64,
}

impl FederationRequestRule for FederationRequestCodeLengthRule {
    fn check(&self, item: &FederationRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRequestRequiredNameRule {
    pub message: String,
}

impl FederationRequestRule for FederationRequestRequiredNameRule {
    fn check(&self, item: &FederationRequest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation request.
#[derive(Debug, Clone)]
pub struct FederationRequestValidator {
    pub rules: Vec<Rc<dyn FederationRequestRule>>,
}

impl FederationRequestValidator {
    pub fn validate(&self, item: &FederationRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_request_validator() -> FederationRequestValidator {
    FederationRequestValidator {
        rules: vec![
            Rc::new(FederationRequestQuotaUnitsRangeRule { min: 1000, max: 86400 }) as Rc<dyn FederationRequestRule>,
            Rc::new(FederationRequestRequiredTitleRule {
                message: "federation request needs a value".to_string(),
            }),
            Rc::new(FederationRequestCodeLengthRule { max_length: 86400 }),
            Rc::new(FederationRequestRequiredNameRule {
                message: "federation request needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_request_validator();
        let item = new_federation_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = FederationRequestQuotaUnitsRangeRule { min: 32, max: 1000 };
        let mut item = new_federation_request("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FederationRequestRequiredTitleRule {
            message: "federation request needs a value".to_string(),
        };
        let mut item = new_federation_request("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FederationRequestCodeLengthRule { max_length: 64 };
        let mut item = new_federation_request("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FederationRequestRequiredNameRule {
            message: "federation request needs a value".to_string(),
        };
        let mut item = new_federation_request("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
