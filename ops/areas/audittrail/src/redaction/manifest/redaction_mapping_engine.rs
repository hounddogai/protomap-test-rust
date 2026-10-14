use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the redaction mapping calculation.
pub trait RedactionMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionMappingAdjuster {
    pub cap: f64,
}

impl RedactionMappingFactor for CapRedactionMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionMappingAdjuster {
    pub minimum: f64,
}

impl RedactionMappingFactor for MinimumRedactionMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionMappingAdjuster {
    pub step: f64,
}

impl RedactionMappingFactor for RoundingRedactionMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionMappingAdjuster {
    pub percent: f64,
}

impl RedactionMappingFactor for PercentRedactionMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRedactionMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionMappingFactor for TitleRedactionMappingAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RedactionMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionMappingFactor for TieredRedactionMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the redaction mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionMappingEngine {
    pub steps: Vec<Rc<dyn RedactionMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionMappingEngine {
    pub fn evaluate(&mut self, item: &RedactionMapping) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[RedactionMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_redaction_mapping_engine() -> RedactionMappingEngine {
    RedactionMappingEngine {
        steps: vec![
            Rc::new(CapRedactionMappingAdjuster { cap: 5793.0 }) as Rc<dyn RedactionMappingFactor>,
            Rc::new(MinimumRedactionMappingAdjuster { minimum: 48.4 }),
            Rc::new(RoundingRedactionMappingAdjuster { step: 0.05 }),
            Rc::new(PercentRedactionMappingAdjuster { percent: -15.0 }),
            Rc::new(TitleRedactionMappingAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.84), ("central".to_string(), 0.83)]),
            }),
            Rc::new(TieredRedactionMappingAdjuster {
                bounds: vec![1340.0, 7203.0],
                rates: vec![0.89, 1.01],
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 34388.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_mapping_engine();
        let item = new_redaction_mapping("code-384", "title-449");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
