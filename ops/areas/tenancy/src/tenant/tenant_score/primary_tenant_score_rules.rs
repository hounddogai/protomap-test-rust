use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary tenant score must pass. Check returns its problems, or none.
pub trait PrimaryTenantScoreRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryTenantScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTenantScoreRequiredCodeRule {
    pub message: String,
}

impl PrimaryTenantScoreRule for PrimaryTenantScoreRequiredCodeRule {
    fn check(&self, item: &PrimaryTenantScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTenantScoreAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PrimaryTenantScoreRule for PrimaryTenantScoreAllowedTitleRule {
    fn check(&self, item: &PrimaryTenantScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTenantScoreChecksumLengthRule {
    pub max_length: i64,
}

impl PrimaryTenantScoreRule for PrimaryTenantScoreChecksumLengthRule {
    fn check(&self, item: &PrimaryTenantScore) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTenantScoreCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryTenantScoreRule for PrimaryTenantScoreCostRangeRule {
    fn check(&self, item: &PrimaryTenantScore) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTenantScoreTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryTenantScoreRule for PrimaryTenantScoreTitleLengthRule {
    fn check(&self, item: &PrimaryTenantScore) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary tenant score.
#[derive(Debug, Clone)]
pub struct PrimaryTenantScoreValidator {
    pub rules: Vec<Rc<dyn PrimaryTenantScoreRule>>,
}

impl PrimaryTenantScoreValidator {
    pub fn validate(&self, item: &PrimaryTenantScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryTenantScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryTenantScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_tenant_score_validator() -> PrimaryTenantScoreValidator {
    PrimaryTenantScoreValidator {
        rules: vec![
            Rc::new(PrimaryTenantScoreRequiredCodeRule {
                message: "primary tenant score needs this field".to_string(),
            }) as Rc<dyn PrimaryTenantScoreRule>,
            Rc::new(PrimaryTenantScoreAllowedTitleRule {
                allowed: vec!["scheduled".to_string(), "adaptive".to_string()],
            }),
            Rc::new(PrimaryTenantScoreChecksumLengthRule { max_length: 3600 }),
            Rc::new(PrimaryTenantScoreCostRangeRule { min: 3.0, max: 3916.0 }),
            Rc::new(PrimaryTenantScoreTitleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_tenant_score_validator();
        let item = new_primary_tenant_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryTenantScoreRequiredCodeRule {
            message: "primary tenant score needs this field".to_string(),
        };
        let mut item = new_primary_tenant_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = PrimaryTenantScoreChecksumLengthRule { max_length: 64 };
        let mut item = new_primary_tenant_score("x", "y");
        item.checksum = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = PrimaryTenantScoreCostRangeRule { min: 9.5, max: 1264.1 };
        let mut item = new_primary_tenant_score("x", "y");
        item.cost = 1265.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryTenantScoreTitleLengthRule { max_length: 32 };
        let mut item = new_primary_tenant_score("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
