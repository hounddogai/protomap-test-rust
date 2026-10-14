use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional nonce preference calculation.
pub trait RegionalNoncePreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalNoncePreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalNoncePreferenceAdjuster {
    pub cap: f64,
}

impl RegionalNoncePreferenceFactor for CapRegionalNoncePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalNoncePreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRegionalNoncePreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalNoncePreferenceFactor for CountryRegionalNoncePreferenceAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RegionalNoncePreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalNoncePreferenceAdjuster {
    pub minimum: f64,
}

impl RegionalNoncePreferenceFactor for MinimumRegionalNoncePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalNoncePreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalNoncePreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalNoncePreferenceFactor for FlaggedRegionalNoncePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalNoncePreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalNoncePreferenceCapacityOffsetAdjuster {
    pub weight: f64,
}

impl RegionalNoncePreferenceFactor for RegionalNoncePreferenceCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalNoncePreference, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

/// Runs the regional nonce preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalNoncePreferenceEngine {
    pub steps: Vec<Rc<dyn RegionalNoncePreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalNoncePreferenceEngine {
    pub fn evaluate(&mut self, item: &RegionalNoncePreference) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalNoncePreferenceFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalNoncePreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_nonce_preference_engine(flags: &flags::Registry) -> RegionalNoncePreferenceEngine {
    RegionalNoncePreferenceEngine {
        steps: vec![
            Rc::new(CapRegionalNoncePreferenceAdjuster { cap: 49531.0 }) as Rc<dyn RegionalNoncePreferenceFactor>,
            Rc::new(CountryRegionalNoncePreferenceAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.32),
                    ("west".to_string(), 1.14),
                    ("east".to_string(), 1.1),
                ]),
            }),
            Rc::new(MinimumRegionalNoncePreferenceAdjuster { minimum: 6.9 }),
            Rc::new(FlaggedRegionalNoncePreferenceAdjuster {
                flags: flags.clone(),
                flag: "regional-nonce-preference-primary".to_string(),
                percent: -5.8,
            }),
            Rc::new(RegionalNoncePreferenceCapacityOffsetAdjuster { weight: 1.95 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 76652.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_nonce_preference_engine(&flags::new_registry());
        let item = new_regional_nonce_preference("code-93", "title-286");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
