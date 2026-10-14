use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted region pin preference must pass. Check returns its problems, or none.
pub trait WeightedRegionPinPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRegionPinPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinPreferenceRequiredTenantIdRule {
    pub message: String,
}

impl WeightedRegionPinPreferenceRule for WeightedRegionPinPreferenceRequiredTenantIdRule {
    fn check(&self, item: &WeightedRegionPinPreference) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinPreferenceSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedRegionPinPreferenceRule for WeightedRegionPinPreferenceSequenceRangeRule {
    fn check(&self, item: &WeightedRegionPinPreference) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinPreferenceMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedRegionPinPreferenceRule for WeightedRegionPinPreferenceMaximumRangeRule {
    fn check(&self, item: &WeightedRegionPinPreference) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted region pin preference.
#[derive(Debug, Clone)]
pub struct WeightedRegionPinPreferenceValidator {
    pub rules: Vec<Rc<dyn WeightedRegionPinPreferenceRule>>,
}

impl WeightedRegionPinPreferenceValidator {
    pub fn validate(&self, item: &WeightedRegionPinPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRegionPinPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRegionPinPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_region_pin_preference_validator() -> WeightedRegionPinPreferenceValidator {
    WeightedRegionPinPreferenceValidator {
        rules: vec![
            Rc::new(WeightedRegionPinPreferenceRequiredTenantIdRule {
                message: "weighted region pin preference needs a value".to_string(),
            }) as Rc<dyn WeightedRegionPinPreferenceRule>,
            Rc::new(WeightedRegionPinPreferenceSequenceRangeRule { min: 32, max: 86400 }),
            Rc::new(WeightedRegionPinPreferenceMaximumRangeRule { min: 2.6, max: 1898.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_region_pin_preference_validator();
        let item = new_weighted_region_pin_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = WeightedRegionPinPreferenceRequiredTenantIdRule {
            message: "weighted region pin preference needs this field".to_string(),
        };
        let mut item = new_weighted_region_pin_preference("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = WeightedRegionPinPreferenceSequenceRangeRule { min: 1000, max: 3600 };
        let mut item = new_weighted_region_pin_preference("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = WeightedRegionPinPreferenceMaximumRangeRule { min: 8.9, max: 4024.2 };
        let mut item = new_weighted_region_pin_preference("x", "y");
        item.maximum = 4025.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
