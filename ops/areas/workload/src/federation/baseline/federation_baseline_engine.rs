use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation baseline calculation.
pub trait FederationBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationBaselineStep for FlaggedFederationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationBaselineAdjuster {
    pub minimum: f64,
}

impl FederationBaselineStep for MinimumFederationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationBaselineAdjuster {
    pub step: f64,
}

impl FederationBaselineStep for RoundingFederationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationBaselineAdjuster {
    pub percent: f64,
}

impl FederationBaselineStep for PercentFederationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationBaselineStep for TieredFederationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryFederationBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationBaselineStep for CountryFederationBaselineAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &FederationBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the federation baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationBaselineEngine {
    pub steps: Vec<Rc<dyn FederationBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationBaselineEngine {
    pub fn compute(&mut self, item: &FederationBaseline) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[FederationBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_federation_baseline_engine(flags: &flags::Registry) -> FederationBaselineEngine {
    FederationBaselineEngine {
        steps: vec![
            Rc::new(FlaggedFederationBaselineAdjuster {
                flags: flags.clone(),
                flag: "federation-baseline-lenient".to_string(),
                percent: 9.5,
            }) as Rc<dyn FederationBaselineStep>,
            Rc::new(MinimumFederationBaselineAdjuster { minimum: 18.9 }),
            Rc::new(RoundingFederationBaselineAdjuster { step: 0.25 }),
            Rc::new(PercentFederationBaselineAdjuster { percent: 11.5 }),
            Rc::new(TieredFederationBaselineAdjuster {
                bounds: vec![1145.0, 3968.0, 4445.0, 5387.0],
                rates: vec![0.84, 0.93, 0.82, 0.87],
            }),
            Rc::new(CountryFederationBaselineAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.9),
                    ("central".to_string(), 0.71),
                    ("north".to_string(), 0.88),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 30961.3,
    }
}
