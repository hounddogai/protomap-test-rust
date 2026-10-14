use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the postmortem manifest calculation.
pub trait PostmortemManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemManifestAdjuster {
    pub step: f64,
}

impl PostmortemManifestStep for RoundingPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemManifestAdjuster {
    pub cap: f64,
}

impl PostmortemManifestStep for CapPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemManifestStep for FlaggedPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPostmortemManifestAdjuster {
    pub percent: f64,
}

impl PostmortemManifestStep for PercentPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PostmortemManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemManifestStep for TieredPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemManifestMinimumOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemManifestStep for PostmortemManifestMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemManifest, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the postmortem manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemManifestEngine {
    pub steps: Vec<Rc<dyn PostmortemManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemManifestEngine {
    pub fn run(&mut self, item: &PostmortemManifest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemManifestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_postmortem_manifest_engine(flags: &flags::Registry) -> PostmortemManifestEngine {
    PostmortemManifestEngine {
        steps: vec![
            Rc::new(RoundingPostmortemManifestAdjuster { step: 0.5 }) as Rc<dyn PostmortemManifestStep>,
            Rc::new(CapPostmortemManifestAdjuster { cap: 47510.0 }),
            Rc::new(FlaggedPostmortemManifestAdjuster {
                flags: flags.clone(),
                flag: "postmortem-manifest-global".to_string(),
                percent: 5.7,
            }),
            Rc::new(PercentPostmortemManifestAdjuster { percent: -4.9 }),
            Rc::new(TieredPostmortemManifestAdjuster {
                bounds: vec![959.0, 8319.0, 9980.0],
                rates: vec![0.87, 1.07, 0.83],
            }),
            Rc::new(PostmortemManifestMinimumOffsetAdjuster { weight: 0.35 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 77426.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_manifest_engine(&flags::new_registry());
        let item = new_postmortem_manifest("code-120", "title-583");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
