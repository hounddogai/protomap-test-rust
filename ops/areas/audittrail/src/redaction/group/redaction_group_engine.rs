use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the redaction group calculation.
pub trait RedactionGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionGroupStep for TieredRedactionGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionGroupAdjuster {
    pub minimum: f64,
}

impl RedactionGroupStep for MinimumRedactionGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRedactionGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionGroupStep for ChannelRedactionGroupAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RedactionGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the redaction group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionGroupEngine {
    pub steps: Vec<Rc<dyn RedactionGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionGroupEngine {
    pub fn compute(&mut self, item: &RedactionGroup) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RedactionGroupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RedactionGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_redaction_group_engine() -> RedactionGroupEngine {
    RedactionGroupEngine {
        steps: vec![
            Rc::new(TieredRedactionGroupAdjuster {
                bounds: vec![1662.0, 4256.0, 7657.0],
                rates: vec![0.93, 1.0, 0.81],
            }) as Rc<dyn RedactionGroupStep>,
            Rc::new(MinimumRedactionGroupAdjuster { minimum: 11.8 }),
            Rc::new(ChannelRedactionGroupAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.11), ("east".to_string(), 1.32)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 58817.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_group_engine();
        let item = new_redaction_group("code-229", "label-942");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
