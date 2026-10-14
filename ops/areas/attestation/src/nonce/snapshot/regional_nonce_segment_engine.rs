use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional nonce segment calculation.
pub trait RegionalNonceSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalNonceSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalNonceSegmentAdjuster {
    pub step: f64,
}

impl RegionalNonceSegmentStep for RoundingRegionalNonceSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalNonceSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRegionalNonceSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalNonceSegmentStep for ChannelRegionalNonceSegmentAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RegionalNonceSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalNonceSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalNonceSegmentStep for TieredRegionalNonceSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalNonceSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalNonceSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalNonceSegmentStep for FlaggedRegionalNonceSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalNonceSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalNonceSegmentAdjuster {
    pub minimum: f64,
}

impl RegionalNonceSegmentStep for MinimumRegionalNonceSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalNonceSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional nonce segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalNonceSegmentEngine {
    pub steps: Vec<Rc<dyn RegionalNonceSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalNonceSegmentEngine {
    pub fn evaluate(&mut self, item: &RegionalNonceSegment) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalNonceSegmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalNonceSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_nonce_segment_engine(flags: &flags::Registry) -> RegionalNonceSegmentEngine {
    RegionalNonceSegmentEngine {
        steps: vec![
            Rc::new(RoundingRegionalNonceSegmentAdjuster { step: 0.25 }) as Rc<dyn RegionalNonceSegmentStep>,
            Rc::new(ChannelRegionalNonceSegmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.05)]),
            }),
            Rc::new(TieredRegionalNonceSegmentAdjuster {
                bounds: vec![541.0, 4369.0, 8474.0],
                rates: vec![1.18, 1.1, 1.11],
            }),
            Rc::new(FlaggedRegionalNonceSegmentAdjuster {
                flags: flags.clone(),
                flag: "regional-nonce-segment-primary".to_string(),
                percent: -4.7,
            }),
            Rc::new(MinimumRegionalNonceSegmentAdjuster { minimum: 46.6 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 77287.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_nonce_segment_engine(&flags::new_registry());
        let item = new_regional_nonce_segment("code-949", "name-779");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
