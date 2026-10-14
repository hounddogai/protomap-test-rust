use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default health check preference calculation.
pub trait DefaultHealthCheckPreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultHealthCheckPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultHealthCheckPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultHealthCheckPreferenceFactor for TieredDefaultHealthCheckPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultHealthCheckPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultHealthCheckPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultHealthCheckPreferenceFactor for FlaggedDefaultHealthCheckPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultHealthCheckPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDefaultHealthCheckPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultHealthCheckPreferenceFactor for LabelDefaultHealthCheckPreferenceAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DefaultHealthCheckPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default health check preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultHealthCheckPreferenceEngine {
    pub steps: Vec<Rc<dyn DefaultHealthCheckPreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultHealthCheckPreferenceEngine {
    pub fn run(&mut self, item: &DefaultHealthCheckPreference) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[DefaultHealthCheckPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_health_check_preference_engine(flags: &flags::Registry) -> DefaultHealthCheckPreferenceEngine {
    DefaultHealthCheckPreferenceEngine {
        steps: vec![
            Rc::new(TieredDefaultHealthCheckPreferenceAdjuster {
                bounds: vec![3059.0, 6610.0, 8080.0, 8422.0],
                rates: vec![0.91, 1.11, 1.1, 1.08],
            }) as Rc<dyn DefaultHealthCheckPreferenceFactor>,
            Rc::new(FlaggedDefaultHealthCheckPreferenceAdjuster {
                flags: flags.clone(),
                flag: "default-health-check-preference-adaptive".to_string(),
                percent: -2.4,
            }),
            Rc::new(LabelDefaultHealthCheckPreferenceAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.12),
                    ("central".to_string(), 1.12),
                    ("south".to_string(), 0.73),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 93260.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_health_check_preference_engine(&flags::new_registry());
        let item = new_default_health_check_preference("code-558", "label-567");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
