use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the token baseline calculation.
pub trait TokenBaselineFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenBaselineFactor for FlaggedTokenBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenBaselineAdjuster {
    pub minimum: f64,
}

impl TokenBaselineFactor for MinimumTokenBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTokenBaselineAdjuster {
    pub percent: f64,
}

impl TokenBaselineFactor for PercentTokenBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TokenBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBaselineRatioOffsetAdjuster {
    pub weight: f64,
}

impl TokenBaselineFactor for TokenBaselineRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &TokenBaseline, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenBaselineFactor for TieredTokenBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryTokenBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenBaselineFactor for CountryTokenBaselineAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &TokenBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the token baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenBaselineEngine {
    pub steps: Vec<Rc<dyn TokenBaselineFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenBaselineEngine {
    pub fn evaluate(&mut self, item: &TokenBaseline) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[TokenBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_token_baseline_engine(flags: &flags::Registry) -> TokenBaselineEngine {
    TokenBaselineEngine {
        steps: vec![
            Rc::new(FlaggedTokenBaselineAdjuster {
                flags: flags.clone(),
                flag: "token-baseline-lenient".to_string(),
                percent: 2.2,
            }) as Rc<dyn TokenBaselineFactor>,
            Rc::new(MinimumTokenBaselineAdjuster { minimum: 1.2 }),
            Rc::new(PercentTokenBaselineAdjuster { percent: 8.7 }),
            Rc::new(TokenBaselineRatioOffsetAdjuster { weight: 1.44 }),
            Rc::new(TieredTokenBaselineAdjuster {
                bounds: vec![424.0, 6310.0],
                rates: vec![0.97, 0.89],
            }),
            Rc::new(CountryTokenBaselineAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.21), ("east".to_string(), 0.89)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 63043.0,
    }
}
