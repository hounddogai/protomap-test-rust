use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered quote segment calculation.
pub trait TieredQuoteSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredQuoteSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTieredQuoteSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredQuoteSegmentFactor for LabelTieredQuoteSegmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TieredQuoteSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredQuoteSegmentAdjuster {
    pub step: f64,
}

impl TieredQuoteSegmentFactor for RoundingTieredQuoteSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredQuoteSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredQuoteSegmentAdjuster {
    pub cap: f64,
}

impl TieredQuoteSegmentFactor for CapTieredQuoteSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredQuoteSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredQuoteSegmentAdjuster {
    pub minimum: f64,
}

impl TieredQuoteSegmentFactor for MinimumTieredQuoteSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredQuoteSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredQuoteSegmentAdjuster {
    pub percent: f64,
}

impl TieredQuoteSegmentFactor for PercentTieredQuoteSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredQuoteSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredQuoteSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredQuoteSegmentFactor for TieredTieredQuoteSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredQuoteSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered quote segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredQuoteSegmentEngine {
    pub steps: Vec<Rc<dyn TieredQuoteSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredQuoteSegmentEngine {
    pub fn compute(&mut self, item: &TieredQuoteSegment) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredQuoteSegmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredQuoteSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_quote_segment_engine() -> TieredQuoteSegmentEngine {
    TieredQuoteSegmentEngine {
        steps: vec![
            Rc::new(LabelTieredQuoteSegmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.99), ("north".to_string(), 0.91)]),
            }) as Rc<dyn TieredQuoteSegmentFactor>,
            Rc::new(RoundingTieredQuoteSegmentAdjuster { step: 0.5 }),
            Rc::new(CapTieredQuoteSegmentAdjuster { cap: 10274.0 }),
            Rc::new(MinimumTieredQuoteSegmentAdjuster { minimum: 41.8 }),
            Rc::new(PercentTieredQuoteSegmentAdjuster { percent: -1.0 }),
            Rc::new(TieredTieredQuoteSegmentAdjuster {
                bounds: vec![2461.0, 6216.0],
                rates: vec![1.1, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 66920.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_quote_segment_engine();
        let item = new_tiered_quote_segment("code-665", "label-35");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
