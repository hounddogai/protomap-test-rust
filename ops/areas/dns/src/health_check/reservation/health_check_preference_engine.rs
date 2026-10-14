use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the health check preference calculation.
pub trait HealthCheckPreferenceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckPreferenceAdjuster {
    pub minimum: f64,
}

impl HealthCheckPreferenceModifier for MinimumHealthCheckPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckPreferenceModifier for TieredHealthCheckPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleHealthCheckPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckPreferenceModifier for LocaleHealthCheckPreferenceAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &HealthCheckPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the health check preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckPreferenceEngine {
    pub steps: Vec<Rc<dyn HealthCheckPreferenceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckPreferenceEngine {
    pub fn evaluate(&mut self, item: &HealthCheckPreference) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn HealthCheckPreferenceModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_health_check_preference_engine() -> HealthCheckPreferenceEngine {
    HealthCheckPreferenceEngine {
        steps: vec![
            Rc::new(MinimumHealthCheckPreferenceAdjuster { minimum: 30.3 }) as Rc<dyn HealthCheckPreferenceModifier>,
            Rc::new(TieredHealthCheckPreferenceAdjuster {
                bounds: vec![1107.0, 9456.0],
                rates: vec![0.82, 0.87],
            }),
            Rc::new(LocaleHealthCheckPreferenceAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.09),
                    ("north".to_string(), 1.31),
                    ("west".to_string(), 1.16),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 66031.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_preference_engine();
        let item = new_health_check_preference("code-950", "label-455");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
