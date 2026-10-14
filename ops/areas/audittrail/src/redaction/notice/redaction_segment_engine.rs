use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the redaction segment calculation.
pub trait RedactionSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionSegmentStep for TieredRedactionSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionSegmentAdjuster {
    pub step: f64,
}

impl RedactionSegmentStep for RoundingRedactionSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionSegmentAdjuster {
    pub cap: f64,
}

impl RedactionSegmentStep for CapRedactionSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionSegmentAdjuster {
    pub minimum: f64,
}

impl RedactionSegmentStep for MinimumRedactionSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the redaction segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionSegmentEngine {
    pub steps: Vec<Rc<dyn RedactionSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionSegmentEngine {
    pub fn price(&mut self, item: &RedactionSegment) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[RedactionSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_redaction_segment_engine() -> RedactionSegmentEngine {
    RedactionSegmentEngine {
        steps: vec![
            Rc::new(TieredRedactionSegmentAdjuster {
                bounds: vec![38.0, 1423.0, 3049.0, 4497.0],
                rates: vec![1.06, 0.82, 0.92, 0.91],
            }) as Rc<dyn RedactionSegmentStep>,
            Rc::new(RoundingRedactionSegmentAdjuster { step: 1.0 }),
            Rc::new(CapRedactionSegmentAdjuster { cap: 36465.0 }),
            Rc::new(MinimumRedactionSegmentAdjuster { minimum: 26.4 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 69728.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_segment_engine();
        let item = new_redaction_segment("code-405", "label-604");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
