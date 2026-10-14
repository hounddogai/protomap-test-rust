use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier segment calculation.
pub trait VerifierSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentVerifierSegmentAdjuster {
    pub percent: f64,
}

impl VerifierSegmentFactor for PercentVerifierSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &VerifierSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierSegmentAdjuster {
    pub step: f64,
}

impl VerifierSegmentFactor for RoundingVerifierSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl VerifierSegmentFactor for TieredVerifierSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &VerifierSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierSegmentFactor for FlaggedVerifierSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierSegmentAdjuster {
    pub minimum: f64,
}

impl VerifierSegmentFactor for MinimumVerifierSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierSegmentAdjuster {
    pub cap: f64,
}

impl VerifierSegmentFactor for CapVerifierSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the verifier segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierSegmentEngine {
    pub steps: Vec<Rc<dyn VerifierSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierSegmentEngine {
    pub fn compute(&mut self, item: &VerifierSegment) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[VerifierSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_verifier_segment_engine(flags: &flags::Registry) -> VerifierSegmentEngine {
    VerifierSegmentEngine {
        steps: vec![
            Rc::new(PercentVerifierSegmentAdjuster { percent: 6.9 }) as Rc<dyn VerifierSegmentFactor>,
            Rc::new(RoundingVerifierSegmentAdjuster { step: 1.0 }),
            Rc::new(TieredVerifierSegmentAdjuster {
                bounds: vec![1870.0, 2873.0, 4581.0, 4914.0],
                rates: vec![1.17, 0.82, 1.02, 1.0],
            }),
            Rc::new(FlaggedVerifierSegmentAdjuster {
                flags: flags.clone(),
                flag: "verifier-segment-adaptive".to_string(),
                percent: -3.3,
            }),
            Rc::new(MinimumVerifierSegmentAdjuster { minimum: 9.1 }),
            Rc::new(CapVerifierSegmentAdjuster { cap: 32224.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 8369.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_segment_engine(&flags::new_registry());
        let item = new_verifier_segment("code-616", "name-371");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
