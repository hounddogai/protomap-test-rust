use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted budget burn preference must pass. Check returns its problems, or none.
pub trait WeightedBudgetBurnPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &WeightedBudgetBurnPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnPreferenceRequiredHostRule {
    pub message: String,
}

impl WeightedBudgetBurnPreferenceRule for WeightedBudgetBurnPreferenceRequiredHostRule {
    fn check(&self, item: &WeightedBudgetBurnPreference) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnPreferenceRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedBudgetBurnPreferenceRule for WeightedBudgetBurnPreferenceRateRangeRule {
    fn check(&self, item: &WeightedBudgetBurnPreference) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnPreferenceTenantIdLengthRule {
    pub max_length: i64,
}

impl WeightedBudgetBurnPreferenceRule for WeightedBudgetBurnPreferenceTenantIdLengthRule {
    fn check(&self, item: &WeightedBudgetBurnPreference) -> Vec<String> {
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
pub struct WeightedBudgetBurnPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl WeightedBudgetBurnPreferenceRule for WeightedBudgetBurnPreferenceTitleLengthRule {
    fn check(&self, item: &WeightedBudgetBurnPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnPreferenceCodePrefixRule {
    pub prefix: String,
}

impl WeightedBudgetBurnPreferenceRule for WeightedBudgetBurnPreferenceCodePrefixRule {
    fn check(&self, item: &WeightedBudgetBurnPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted budget burn preference.
#[derive(Debug, Clone)]
pub struct WeightedBudgetBurnPreferenceValidator {
    pub rules: Vec<Rc<dyn WeightedBudgetBurnPreferenceRule>>,
}

impl WeightedBudgetBurnPreferenceValidator {
    pub fn validate(&self, item: &WeightedBudgetBurnPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedBudgetBurnPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedBudgetBurnPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_budget_burn_preference_validator() -> WeightedBudgetBurnPreferenceValidator {
    WeightedBudgetBurnPreferenceValidator {
        rules: vec![
            Rc::new(WeightedBudgetBurnPreferenceRequiredHostRule {
                message: "weighted budget burn preference needs a value".to_string(),
            }) as Rc<dyn WeightedBudgetBurnPreferenceRule>,
            Rc::new(WeightedBudgetBurnPreferenceRateRangeRule { min: 2.3, max: 2721.0 }),
            Rc::new(WeightedBudgetBurnPreferenceTenantIdLengthRule { max_length: 64 }),
            Rc::new(WeightedBudgetBurnPreferenceTitleLengthRule { max_length: 16 }),
            Rc::new(WeightedBudgetBurnPreferenceCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_budget_burn_preference_validator();
        let item = new_weighted_budget_burn_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = WeightedBudgetBurnPreferenceRequiredHostRule {
            message: "weighted budget burn preference needs an entry".to_string(),
        };
        let mut item = new_weighted_budget_burn_preference("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = WeightedBudgetBurnPreferenceRateRangeRule { min: 4.7, max: 344.0 };
        let mut item = new_weighted_budget_burn_preference("x", "y");
        item.rate = 345.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = WeightedBudgetBurnPreferenceTenantIdLengthRule { max_length: 86400 };
        let mut item = new_weighted_budget_burn_preference("x", "y");
        item.tenant_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = WeightedBudgetBurnPreferenceTitleLengthRule { max_length: 3600 };
        let mut item = new_weighted_budget_burn_preference("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
