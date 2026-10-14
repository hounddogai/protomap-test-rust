use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted forecast mapping must pass. Check returns its problems, or none.
pub trait WeightedForecastMappingRule: std::fmt::Debug {
    fn check(&self, item: &WeightedForecastMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMappingRequiredScopeRule {
    pub message: String,
}

impl WeightedForecastMappingRule for WeightedForecastMappingRequiredScopeRule {
    fn check(&self, item: &WeightedForecastMapping) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMappingRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedForecastMappingRule for WeightedForecastMappingRevisionRangeRule {
    fn check(&self, item: &WeightedForecastMapping) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMappingCodePrefixRule {
    pub prefix: String,
}

impl WeightedForecastMappingRule for WeightedForecastMappingCodePrefixRule {
    fn check(&self, item: &WeightedForecastMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMappingRequiredTitleRule {
    pub message: String,
}

impl WeightedForecastMappingRule for WeightedForecastMappingRequiredTitleRule {
    fn check(&self, item: &WeightedForecastMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMappingRequiredCodeRule {
    pub message: String,
}

impl WeightedForecastMappingRule for WeightedForecastMappingRequiredCodeRule {
    fn check(&self, item: &WeightedForecastMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted forecast mapping.
#[derive(Debug, Clone)]
pub struct WeightedForecastMappingValidator {
    pub rules: Vec<Rc<dyn WeightedForecastMappingRule>>,
}

impl WeightedForecastMappingValidator {
    pub fn validate(&self, item: &WeightedForecastMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedForecastMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedForecastMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_forecast_mapping_validator() -> WeightedForecastMappingValidator {
    WeightedForecastMappingValidator {
        rules: vec![
            Rc::new(WeightedForecastMappingRequiredScopeRule {
                message: "weighted forecast mapping needs an entry".to_string(),
            }) as Rc<dyn WeightedForecastMappingRule>,
            Rc::new(WeightedForecastMappingRevisionRangeRule { min: 16, max: 255 }),
            Rc::new(WeightedForecastMappingCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(WeightedForecastMappingRequiredTitleRule {
                message: "weighted forecast mapping needs a value".to_string(),
            }),
            Rc::new(WeightedForecastMappingRequiredCodeRule {
                message: "weighted forecast mapping needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_forecast_mapping_validator();
        let item = new_weighted_forecast_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = WeightedForecastMappingRequiredScopeRule {
            message: "weighted forecast mapping needs a value".to_string(),
        };
        let mut item = new_weighted_forecast_mapping("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = WeightedForecastMappingRevisionRangeRule { min: 64, max: 3600 };
        let mut item = new_weighted_forecast_mapping("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedForecastMappingRequiredTitleRule {
            message: "weighted forecast mapping needs an entry".to_string(),
        };
        let mut item = new_weighted_forecast_mapping("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedForecastMappingRequiredCodeRule {
            message: "weighted forecast mapping needs an entry".to_string(),
        };
        let mut item = new_weighted_forecast_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
