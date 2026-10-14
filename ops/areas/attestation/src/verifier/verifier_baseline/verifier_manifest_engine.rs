use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier manifest calculation.
pub trait VerifierManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierManifestAdjuster {
    pub cap: f64,
}

impl VerifierManifestStep for CapVerifierManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierManifestStep for FlaggedVerifierManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierManifestAdjuster {
    pub minimum: f64,
}

impl VerifierManifestStep for MinimumVerifierManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentVerifierManifestAdjuster {
    pub percent: f64,
}

impl VerifierManifestStep for PercentVerifierManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &VerifierManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierManifestAdjuster {
    pub step: f64,
}

impl VerifierManifestStep for RoundingVerifierManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the verifier manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierManifestEngine {
    pub steps: Vec<Rc<dyn VerifierManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierManifestEngine {
    pub fn run(&mut self, item: &VerifierManifest) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierManifestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[VerifierManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_verifier_manifest_engine(flags: &flags::Registry) -> VerifierManifestEngine {
    VerifierManifestEngine {
        steps: vec![
            Rc::new(CapVerifierManifestAdjuster { cap: 17437.0 }) as Rc<dyn VerifierManifestStep>,
            Rc::new(FlaggedVerifierManifestAdjuster {
                flags: flags.clone(),
                flag: "verifier-manifest-pending".to_string(),
                percent: 3.4,
            }),
            Rc::new(MinimumVerifierManifestAdjuster { minimum: 18.9 }),
            Rc::new(PercentVerifierManifestAdjuster { percent: -13.7 }),
            Rc::new(RoundingVerifierManifestAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 63716.4,
    }
}
