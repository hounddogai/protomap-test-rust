use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional policy preference calculation.
pub trait RegionalPolicyPreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalPolicyPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRegionalPolicyPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalPolicyPreferenceFactor for LabelRegionalPolicyPreferenceAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RegionalPolicyPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalPolicyPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalPolicyPreferenceFactor for FlaggedRegionalPolicyPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalPolicyPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalPolicyPreferenceAdjuster {
    pub minimum: f64,
}

impl RegionalPolicyPreferenceFactor for MinimumRegionalPolicyPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalPolicyPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalPolicyPreferenceAdjuster {
    pub percent: f64,
}

impl RegionalPolicyPreferenceFactor for PercentRegionalPolicyPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalPolicyPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalPolicyPreferenceAdjuster {
    pub cap: f64,
}

impl RegionalPolicyPreferenceFactor for CapRegionalPolicyPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalPolicyPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional policy preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalPolicyPreferenceEngine {
    pub steps: Vec<Rc<dyn RegionalPolicyPreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalPolicyPreferenceEngine {
    pub fn price(&mut self, item: &RegionalPolicyPreference) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalPolicyPreferenceFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RegionalPolicyPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_policy_preference_engine(flags: &flags::Registry) -> RegionalPolicyPreferenceEngine {
    RegionalPolicyPreferenceEngine {
        steps: vec![
            Rc::new(LabelRegionalPolicyPreferenceAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.05)]),
            }) as Rc<dyn RegionalPolicyPreferenceFactor>,
            Rc::new(FlaggedRegionalPolicyPreferenceAdjuster {
                flags: flags.clone(),
                flag: "regional-policy-preference-weighted".to_string(),
                percent: -7.1,
            }),
            Rc::new(MinimumRegionalPolicyPreferenceAdjuster { minimum: 27.2 }),
            Rc::new(PercentRegionalPolicyPreferenceAdjuster { percent: -23.7 }),
            Rc::new(CapRegionalPolicyPreferenceAdjuster { cap: 39378.0 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 27271.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_policy_preference_engine(&flags::new_registry());
        let item = new_regional_policy_preference("code-322", "label-351");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
