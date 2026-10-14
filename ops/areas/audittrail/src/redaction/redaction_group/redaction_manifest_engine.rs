use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the redaction manifest calculation.
pub trait RedactionManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionManifestAdjuster {
    pub step: f64,
}

impl RedactionManifestAdjuster for RoundingRedactionManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionManifestAdjuster for FlaggedRedactionManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionManifestAdjuster {
    pub minimum: f64,
}

impl RedactionManifestAdjuster for MinimumRedactionManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the redaction manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionManifestEngine {
    pub steps: Vec<Rc<dyn RedactionManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionManifestEngine {
    pub fn evaluate(&mut self, item: &RedactionManifest) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[RedactionManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_redaction_manifest_engine(flags: &flags::Registry) -> RedactionManifestEngine {
    RedactionManifestEngine {
        steps: vec![
            Rc::new(RoundingRedactionManifestAdjuster { step: 0.5 }) as Rc<dyn RedactionManifestAdjuster>,
            Rc::new(FlaggedRedactionManifestAdjuster {
                flags: flags.clone(),
                flag: "redaction-manifest-batched".to_string(),
                percent: -6.6,
            }),
            Rc::new(MinimumRedactionManifestAdjuster { minimum: 22.0 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 39100.3,
    }
}
