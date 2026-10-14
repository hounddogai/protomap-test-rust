use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled health check preference calculation.
pub trait ScheduledHealthCheckPreferenceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledHealthCheckPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledHealthCheckPreferenceAdjuster {
    pub percent: f64,
}

impl ScheduledHealthCheckPreferenceModifier for PercentScheduledHealthCheckPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledHealthCheckPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeScheduledHealthCheckPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledHealthCheckPreferenceModifier for ScopeScheduledHealthCheckPreferenceAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ScheduledHealthCheckPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledHealthCheckPreferenceAdjuster {
    pub minimum: f64,
}

impl ScheduledHealthCheckPreferenceModifier for MinimumScheduledHealthCheckPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledHealthCheckPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled health check preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledHealthCheckPreferenceEngine {
    pub steps: Vec<Rc<dyn ScheduledHealthCheckPreferenceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledHealthCheckPreferenceEngine {
    pub fn evaluate(&mut self, item: &ScheduledHealthCheckPreference) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledHealthCheckPreferenceModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledHealthCheckPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_health_check_preference_engine() -> ScheduledHealthCheckPreferenceEngine {
    ScheduledHealthCheckPreferenceEngine {
        steps: vec![
            Rc::new(PercentScheduledHealthCheckPreferenceAdjuster { percent: -22.2 })
                as Rc<dyn ScheduledHealthCheckPreferenceModifier>,
            Rc::new(ScopeScheduledHealthCheckPreferenceAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.3), ("south".to_string(), 1.05)]),
            }),
            Rc::new(MinimumScheduledHealthCheckPreferenceAdjuster { minimum: 3.5 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 70764.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_health_check_preference_engine();
        let item = new_scheduled_health_check_preference("code-202", "title-778");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
