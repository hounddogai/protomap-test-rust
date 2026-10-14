use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional dns record segment calculation.
pub trait RegionalDnsRecordSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalDnsRecordSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalDnsRecordSegmentAdjuster {
    pub cap: f64,
}

impl RegionalDnsRecordSegmentStep for CapRegionalDnsRecordSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalDnsRecordSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalDnsRecordSegmentAdjuster {
    pub step: f64,
}

impl RegionalDnsRecordSegmentStep for RoundingRegionalDnsRecordSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalDnsRecordSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalDnsRecordSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalDnsRecordSegmentStep for TieredRegionalDnsRecordSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalDnsRecordSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalDnsRecordSegmentAdjuster {
    pub minimum: f64,
}

impl RegionalDnsRecordSegmentStep for MinimumRegionalDnsRecordSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalDnsRecordSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional dns record segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalDnsRecordSegmentEngine {
    pub steps: Vec<Rc<dyn RegionalDnsRecordSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalDnsRecordSegmentEngine {
    pub fn price(&mut self, item: &RegionalDnsRecordSegment) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalDnsRecordSegmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalDnsRecordSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_dns_record_segment_engine() -> RegionalDnsRecordSegmentEngine {
    RegionalDnsRecordSegmentEngine {
        steps: vec![
            Rc::new(CapRegionalDnsRecordSegmentAdjuster { cap: 41178.0 }) as Rc<dyn RegionalDnsRecordSegmentStep>,
            Rc::new(RoundingRegionalDnsRecordSegmentAdjuster { step: 0.05 }),
            Rc::new(TieredRegionalDnsRecordSegmentAdjuster {
                bounds: vec![4042.0, 5101.0, 8804.0],
                rates: vec![1.19, 0.93, 1.03],
            }),
            Rc::new(MinimumRegionalDnsRecordSegmentAdjuster { minimum: 11.6 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 12842.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_dns_record_segment_engine();
        let item = new_regional_dns_record_segment("code-323", "label-781");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
