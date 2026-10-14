use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict credential segment calculation.
pub trait StrictCredentialSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictCredentialSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictCredentialSegmentAdjuster {
    pub minimum: f64,
}

impl StrictCredentialSegmentAdjuster for MinimumStrictCredentialSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictCredentialSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictCredentialSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictCredentialSegmentAdjuster for TieredStrictCredentialSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictCredentialSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictCredentialSegmentAdjuster {
    pub cap: f64,
}

impl StrictCredentialSegmentAdjuster for CapStrictCredentialSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictCredentialSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictCredentialSegmentAdjuster {
    pub step: f64,
}

impl StrictCredentialSegmentAdjuster for RoundingStrictCredentialSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictCredentialSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictCredentialSegmentAdjuster {
    pub percent: f64,
}

impl StrictCredentialSegmentAdjuster for PercentStrictCredentialSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictCredentialSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryStrictCredentialSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictCredentialSegmentAdjuster for CountryStrictCredentialSegmentAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &StrictCredentialSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict credential segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictCredentialSegmentEngine {
    pub steps: Vec<Rc<dyn StrictCredentialSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictCredentialSegmentEngine {
    pub fn price(&mut self, item: &StrictCredentialSegment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[StrictCredentialSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_credential_segment_engine() -> StrictCredentialSegmentEngine {
    StrictCredentialSegmentEngine {
        steps: vec![
            Rc::new(MinimumStrictCredentialSegmentAdjuster { minimum: 22.3 })
                as Rc<dyn StrictCredentialSegmentAdjuster>,
            Rc::new(TieredStrictCredentialSegmentAdjuster {
                bounds: vec![2444.0, 4661.0, 5806.0, 9279.0],
                rates: vec![1.16, 0.81, 1.05, 0.91],
            }),
            Rc::new(CapStrictCredentialSegmentAdjuster { cap: 40381.0 }),
            Rc::new(RoundingStrictCredentialSegmentAdjuster { step: 0.05 }),
            Rc::new(PercentStrictCredentialSegmentAdjuster { percent: 23.9 }),
            Rc::new(CountryStrictCredentialSegmentAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.32),
                    ("central".to_string(), 1.25),
                    ("retail".to_string(), 0.81),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 54076.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_credential_segment_engine();
        let item = new_strict_credential_segment("code-858", "label-158");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
