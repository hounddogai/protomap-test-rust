use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the redaction baseline calculation.
pub trait RedactionBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionBaselineAdjuster {
    pub percent: f64,
}

impl RedactionBaselineStep for PercentRedactionBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionBaselineAdjuster {
    pub cap: f64,
}

impl RedactionBaselineStep for CapRedactionBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBaselineMarginOffsetAdjuster {
    pub weight: f64,
}

impl RedactionBaselineStep for RedactionBaselineMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionBaseline, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionBaselineStep for TieredRedactionBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceRedactionBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionBaselineStep for NamespaceRedactionBaselineAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &RedactionBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionBaselineAdjuster {
    pub step: f64,
}

impl RedactionBaselineStep for RoundingRedactionBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the redaction baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionBaselineEngine {
    pub steps: Vec<Rc<dyn RedactionBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionBaselineEngine {
    pub fn price(&mut self, item: &RedactionBaseline) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RedactionBaselineStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RedactionBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_redaction_baseline_engine() -> RedactionBaselineEngine {
    RedactionBaselineEngine {
        steps: vec![
            Rc::new(PercentRedactionBaselineAdjuster { percent: -12.1 }) as Rc<dyn RedactionBaselineStep>,
            Rc::new(CapRedactionBaselineAdjuster { cap: 2260.0 }),
            Rc::new(RedactionBaselineMarginOffsetAdjuster { weight: 0.42 }),
            Rc::new(TieredRedactionBaselineAdjuster {
                bounds: vec![1145.0, 5907.0, 9702.0],
                rates: vec![1.15, 1.01, 1.16],
            }),
            Rc::new(NamespaceRedactionBaselineAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.32),
                    ("east".to_string(), 0.76),
                    ("south".to_string(), 1.07),
                ]),
            }),
            Rc::new(RoundingRedactionBaselineAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 23617.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_baseline_engine();
        let item = new_redaction_baseline("code-869", "label-668");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
