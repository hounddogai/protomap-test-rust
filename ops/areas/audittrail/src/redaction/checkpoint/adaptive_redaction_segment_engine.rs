use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive redaction segment calculation.
pub trait AdaptiveRedactionSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRedactionSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceAdaptiveRedactionSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveRedactionSegmentAdjuster for SourceAdaptiveRedactionSegmentAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &AdaptiveRedactionSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveRedactionSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveRedactionSegmentAdjuster for FlaggedAdaptiveRedactionSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveRedactionSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveRedactionSegmentAdjuster {
    pub step: f64,
}

impl AdaptiveRedactionSegmentAdjuster for RoundingAdaptiveRedactionSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveRedactionSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRedactionSegmentFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveRedactionSegmentAdjuster for AdaptiveRedactionSegmentFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveRedactionSegment, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRedactionSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRedactionSegmentAdjuster for TieredAdaptiveRedactionSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRedactionSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveRedactionSegmentAdjuster {
    pub minimum: f64,
}

impl AdaptiveRedactionSegmentAdjuster for MinimumAdaptiveRedactionSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveRedactionSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the adaptive redaction segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRedactionSegmentEngine {
    pub steps: Vec<Rc<dyn AdaptiveRedactionSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRedactionSegmentEngine {
    pub fn evaluate(&mut self, item: &AdaptiveRedactionSegment) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRedactionSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_redaction_segment_engine(flags: &flags::Registry) -> AdaptiveRedactionSegmentEngine {
    AdaptiveRedactionSegmentEngine {
        steps: vec![
            Rc::new(SourceAdaptiveRedactionSegmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.18)]),
            }) as Rc<dyn AdaptiveRedactionSegmentAdjuster>,
            Rc::new(FlaggedAdaptiveRedactionSegmentAdjuster {
                flags: flags.clone(),
                flag: "adaptive-redaction-segment-rolling".to_string(),
                percent: 0.8,
            }),
            Rc::new(RoundingAdaptiveRedactionSegmentAdjuster { step: 1.0 }),
            Rc::new(AdaptiveRedactionSegmentFloorValueOffsetAdjuster { weight: 1.22 }),
            Rc::new(TieredAdaptiveRedactionSegmentAdjuster {
                bounds: vec![218.0, 1650.0, 3303.0, 8853.0],
                rates: vec![1.04, 1.03, 0.95, 1.14],
            }),
            Rc::new(MinimumAdaptiveRedactionSegmentAdjuster { minimum: 11.1 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 56868.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_redaction_segment_engine(&flags::new_registry());
        let item = new_adaptive_redaction_segment("code-363", "name-611");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
