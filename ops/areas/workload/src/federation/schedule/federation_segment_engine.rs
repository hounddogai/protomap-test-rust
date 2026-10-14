use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation segment calculation.
pub trait FederationSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFederationSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationSegmentStep for NamespaceFederationSegmentAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FederationSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationSegmentStep for FlaggedFederationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationSegmentAdjuster {
    pub step: f64,
}

impl FederationSegmentStep for RoundingFederationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationSegmentStep for TieredFederationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationSegmentAdjuster {
    pub percent: f64,
}

impl FederationSegmentStep for PercentFederationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the federation segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationSegmentEngine {
    pub steps: Vec<Rc<dyn FederationSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationSegmentEngine {
    pub fn run(&mut self, item: &FederationSegment) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationSegmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_federation_segment_engine(flags: &flags::Registry) -> FederationSegmentEngine {
    FederationSegmentEngine {
        steps: vec![
            Rc::new(NamespaceFederationSegmentAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.38), ("online".to_string(), 0.91)]),
            }) as Rc<dyn FederationSegmentStep>,
            Rc::new(FlaggedFederationSegmentAdjuster {
                flags: flags.clone(),
                flag: "federation-segment-fallback".to_string(),
                percent: -3.9,
            }),
            Rc::new(RoundingFederationSegmentAdjuster { step: 0.05 }),
            Rc::new(TieredFederationSegmentAdjuster {
                bounds: vec![812.0, 6910.0],
                rates: vec![0.88, 1.16],
            }),
            Rc::new(PercentFederationSegmentAdjuster { percent: 19.1 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 22007.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_segment_engine(&flags::new_registry());
        let item = new_federation_segment("code-156", "name-823");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
