use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the token segment calculation.
pub trait TokenSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenSegmentAdjuster {
    pub minimum: f64,
}

impl TokenSegmentFactor for MinimumTokenSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTokenSegmentAdjuster {
    pub step: f64,
}

impl TokenSegmentFactor for RoundingTokenSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TokenSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenSegmentRetriesOffsetAdjuster {
    pub weight: f64,
}

impl TokenSegmentFactor for TokenSegmentRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &TokenSegment, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTokenSegmentAdjuster {
    pub percent: f64,
}

impl TokenSegmentFactor for PercentTokenSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TokenSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTokenSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenSegmentFactor for ChannelTokenSegmentAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &TokenSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenSegmentAdjuster {
    pub cap: f64,
}

impl TokenSegmentFactor for CapTokenSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the token segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenSegmentEngine {
    pub steps: Vec<Rc<dyn TokenSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenSegmentEngine {
    pub fn compute(&mut self, item: &TokenSegment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TokenSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_token_segment_engine() -> TokenSegmentEngine {
    TokenSegmentEngine {
        steps: vec![
            Rc::new(MinimumTokenSegmentAdjuster { minimum: 27.3 }) as Rc<dyn TokenSegmentFactor>,
            Rc::new(RoundingTokenSegmentAdjuster { step: 0.05 }),
            Rc::new(TokenSegmentRetriesOffsetAdjuster { weight: 1.23 }),
            Rc::new(PercentTokenSegmentAdjuster { percent: 20.7 }),
            Rc::new(ChannelTokenSegmentAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.32),
                    ("east".to_string(), 0.93),
                    ("online".to_string(), 1.29),
                ]),
            }),
            Rc::new(CapTokenSegmentAdjuster { cap: 15078.0 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 15784.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_segment_engine();
        let item = new_token_segment("code-819", "label-311");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
