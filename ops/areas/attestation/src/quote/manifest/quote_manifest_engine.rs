use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the quote manifest calculation.
pub trait QuoteManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeQuoteManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuoteManifestFactor for ScopeQuoteManifestAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &QuoteManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteManifestFactor for FlaggedQuoteManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteManifestAdjuster {
    pub percent: f64,
}

impl QuoteManifestFactor for PercentQuoteManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the quote manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteManifestEngine {
    pub steps: Vec<Rc<dyn QuoteManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteManifestEngine {
    pub fn compute(&mut self, item: &QuoteManifest) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[QuoteManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_quote_manifest_engine(flags: &flags::Registry) -> QuoteManifestEngine {
    QuoteManifestEngine {
        steps: vec![
            Rc::new(ScopeQuoteManifestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.17)]),
            }) as Rc<dyn QuoteManifestFactor>,
            Rc::new(FlaggedQuoteManifestAdjuster {
                flags: flags.clone(),
                flag: "quote-manifest-tiered".to_string(),
                percent: 2.5,
            }),
            Rc::new(PercentQuoteManifestAdjuster { percent: 12.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 3320.3,
    }
}
