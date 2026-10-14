use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict overage bundle calculation.
pub trait StrictOverageBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictOverageBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageBundleLimitOffsetAdjuster {
    pub weight: f64,
}

impl StrictOverageBundleAdjuster for StrictOverageBundleLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &StrictOverageBundle, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictOverageBundleAdjuster {
    pub percent: f64,
}

impl StrictOverageBundleAdjuster for PercentStrictOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictOverageBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictOverageBundleAdjuster {
    pub step: f64,
}

impl StrictOverageBundleAdjuster for RoundingStrictOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictOverageBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictOverageBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictOverageBundleAdjuster for FlaggedStrictOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictOverageBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryStrictOverageBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictOverageBundleAdjuster for CountryStrictOverageBundleAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &StrictOverageBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict overage bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictOverageBundleEngine {
    pub steps: Vec<Rc<dyn StrictOverageBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictOverageBundleEngine {
    pub fn evaluate(&mut self, item: &StrictOverageBundle) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[StrictOverageBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_overage_bundle_engine(flags: &flags::Registry) -> StrictOverageBundleEngine {
    StrictOverageBundleEngine {
        steps: vec![
            Rc::new(StrictOverageBundleLimitOffsetAdjuster { weight: 0.8 }) as Rc<dyn StrictOverageBundleAdjuster>,
            Rc::new(PercentStrictOverageBundleAdjuster { percent: 22.0 }),
            Rc::new(RoundingStrictOverageBundleAdjuster { step: 5.0 }),
            Rc::new(FlaggedStrictOverageBundleAdjuster {
                flags: flags.clone(),
                flag: "strict-overage-bundle-seasonal".to_string(),
                percent: 4.5,
            }),
            Rc::new(CountryStrictOverageBundleAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.01),
                    ("west".to_string(), 1.0),
                    ("south".to_string(), 1.05),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 59147.4,
    }
}
