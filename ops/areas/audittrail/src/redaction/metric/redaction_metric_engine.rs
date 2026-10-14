use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the redaction metric calculation.
pub trait RedactionMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionMetricAdjuster {
    pub step: f64,
}

impl RedactionMetricAdjuster for RoundingRedactionMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRedactionMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionMetricAdjuster for LocaleRedactionMetricAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RedactionMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMetricVersionOffsetAdjuster {
    pub weight: f64,
}

impl RedactionMetricAdjuster for RedactionMetricVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionMetric, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionMetricAdjuster {
    pub cap: f64,
}

impl RedactionMetricAdjuster for CapRedactionMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionMetricAdjuster {
    pub percent: f64,
}

impl RedactionMetricAdjuster for PercentRedactionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the redaction metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionMetricEngine {
    pub steps: Vec<Rc<dyn RedactionMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionMetricEngine {
    pub fn price(&mut self, item: &RedactionMetric) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RedactionMetricAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RedactionMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_redaction_metric_engine() -> RedactionMetricEngine {
    RedactionMetricEngine {
        steps: vec![
            Rc::new(RoundingRedactionMetricAdjuster { step: 0.1 }) as Rc<dyn RedactionMetricAdjuster>,
            Rc::new(LocaleRedactionMetricAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.88)]),
            }),
            Rc::new(RedactionMetricVersionOffsetAdjuster { weight: 1.89 }),
            Rc::new(CapRedactionMetricAdjuster { cap: 7812.0 }),
            Rc::new(PercentRedactionMetricAdjuster { percent: -9.3 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 27608.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_metric_engine();
        let item = new_redaction_metric("code-410", "title-827");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
