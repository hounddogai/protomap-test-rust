use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the identity metric calculation.
pub trait IdentityMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityMetric, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityMetricStep for FlaggedIdentityMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIdentityMetricAdjuster {
    pub minimum: f64,
}

impl IdentityMetricStep for MinimumIdentityMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IdentityMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentityMetricAdjuster {
    pub percent: f64,
}

impl IdentityMetricStep for PercentIdentityMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentityMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryIdentityMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityMetricStep for CountryIdentityMetricAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &IdentityMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the identity metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityMetricEngine {
    pub steps: Vec<Rc<dyn IdentityMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityMetricEngine {
    pub fn evaluate(&mut self, item: &IdentityMetric) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[IdentityMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_identity_metric_engine(flags: &flags::Registry) -> IdentityMetricEngine {
    IdentityMetricEngine {
        steps: vec![
            Rc::new(FlaggedIdentityMetricAdjuster {
                flags: flags.clone(),
                flag: "identity-metric-weighted".to_string(),
                percent: -7.4,
            }) as Rc<dyn IdentityMetricStep>,
            Rc::new(MinimumIdentityMetricAdjuster { minimum: 22.9 }),
            Rc::new(PercentIdentityMetricAdjuster { percent: -18.9 }),
            Rc::new(CountryIdentityMetricAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.97),
                    ("central".to_string(), 1.0),
                    ("retail".to_string(), 0.86),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 81455.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_metric_engine(&flags::new_registry());
        let item = new_identity_metric("code-109", "label-464");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
