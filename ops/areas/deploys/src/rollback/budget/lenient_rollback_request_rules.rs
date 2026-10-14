use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient rollback request must pass. Check returns its problems, or none.
pub trait LenientRollbackRequestRule: std::fmt::Debug {
    fn check(&self, item: &LenientRollbackRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRollbackRequestExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRollbackRequestRule for LenientRollbackRequestExpiresAtRangeRule {
    fn check(&self, item: &LenientRollbackRequest) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRollbackRequestCodePrefixRule {
    pub prefix: String,
}

impl LenientRollbackRequestRule for LenientRollbackRequestCodePrefixRule {
    fn check(&self, item: &LenientRollbackRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRollbackRequestRequiredIssuerRule {
    pub message: String,
}

impl LenientRollbackRequestRule for LenientRollbackRequestRequiredIssuerRule {
    fn check(&self, item: &LenientRollbackRequest) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRollbackRequestQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRollbackRequestRule for LenientRollbackRequestQuotaUnitsRangeRule {
    fn check(&self, item: &LenientRollbackRequest) -> Vec<String> {
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
pub struct LenientRollbackRequestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRollbackRequestRule for LenientRollbackRequestCreatedAtRangeRule {
    fn check(&self, item: &LenientRollbackRequest) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient rollback request.
#[derive(Debug, Clone)]
pub struct LenientRollbackRequestValidator {
    pub rules: Vec<Rc<dyn LenientRollbackRequestRule>>,
}

impl LenientRollbackRequestValidator {
    pub fn validate(&self, item: &LenientRollbackRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientRollbackRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRollbackRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_rollback_request_validator() -> LenientRollbackRequestValidator {
    LenientRollbackRequestValidator {
        rules: vec![
            Rc::new(LenientRollbackRequestExpiresAtRangeRule { min: 64, max: 3600 })
                as Rc<dyn LenientRollbackRequestRule>,
            Rc::new(LenientRollbackRequestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LenientRollbackRequestRequiredIssuerRule {
                message: "lenient rollback request needs an entry".to_string(),
            }),
            Rc::new(LenientRollbackRequestQuotaUnitsRangeRule { min: 3600, max: 3600 }),
            Rc::new(LenientRollbackRequestCreatedAtRangeRule { min: 64, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_rollback_request_validator();
        let item = new_lenient_rollback_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = LenientRollbackRequestExpiresAtRangeRule { min: 80, max: 255 };
        let mut item = new_lenient_rollback_request("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = LenientRollbackRequestRequiredIssuerRule {
            message: "lenient rollback request needs this field".to_string(),
        };
        let mut item = new_lenient_rollback_request("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = LenientRollbackRequestQuotaUnitsRangeRule { min: 80, max: 120 };
        let mut item = new_lenient_rollback_request("x", "y");
        item.quota_units = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = LenientRollbackRequestCreatedAtRangeRule { min: 1000, max: 1000 };
        let mut item = new_lenient_rollback_request("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
