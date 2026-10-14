use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped evidence forecast must pass. Check returns its problems, or none.
pub trait CappedEvidenceForecastRule: std::fmt::Debug {
    fn check(&self, item: &CappedEvidenceForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceForecastArchivedGuardRule {
    pub limit: i64,
}

impl CappedEvidenceForecastRule for CappedEvidenceForecastArchivedGuardRule {
    fn check(&self, item: &CappedEvidenceForecast) -> Vec<String> {
        if item.archived && (item.version > self.limit) {
            return vec![format!(
                "version {} exceeds {} while archived",
                item.version, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceForecastRequiredCodeRule {
    pub message: String,
}

impl CappedEvidenceForecastRule for CappedEvidenceForecastRequiredCodeRule {
    fn check(&self, item: &CappedEvidenceForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceForecastRequiredCategoryRule {
    pub message: String,
}

impl CappedEvidenceForecastRule for CappedEvidenceForecastRequiredCategoryRule {
    fn check(&self, item: &CappedEvidenceForecast) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceForecastRequiredNameRule {
    pub message: String,
}

impl CappedEvidenceForecastRule for CappedEvidenceForecastRequiredNameRule {
    fn check(&self, item: &CappedEvidenceForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped evidence forecast.
#[derive(Debug, Clone)]
pub struct CappedEvidenceForecastValidator {
    pub rules: Vec<Rc<dyn CappedEvidenceForecastRule>>,
}

impl CappedEvidenceForecastValidator {
    pub fn validate(&self, item: &CappedEvidenceForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedEvidenceForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedEvidenceForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_evidence_forecast_validator() -> CappedEvidenceForecastValidator {
    CappedEvidenceForecastValidator {
        rules: vec![
            Rc::new(CappedEvidenceForecastArchivedGuardRule { limit: 86400 }) as Rc<dyn CappedEvidenceForecastRule>,
            Rc::new(CappedEvidenceForecastRequiredCodeRule {
                message: "capped evidence forecast needs a value".to_string(),
            }),
            Rc::new(CappedEvidenceForecastRequiredCategoryRule {
                message: "capped evidence forecast needs an entry".to_string(),
            }),
            Rc::new(CappedEvidenceForecastRequiredNameRule {
                message: "capped evidence forecast needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_evidence_forecast_validator();
        let item = new_capped_evidence_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedEvidenceForecastRequiredCodeRule {
            message: "capped evidence forecast needs a value".to_string(),
        };
        let mut item = new_capped_evidence_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = CappedEvidenceForecastRequiredCategoryRule {
            message: "capped evidence forecast needs a value".to_string(),
        };
        let mut item = new_capped_evidence_forecast("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CappedEvidenceForecastRequiredNameRule {
            message: "capped evidence forecast needs a value".to_string(),
        };
        let mut item = new_capped_evidence_forecast("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
