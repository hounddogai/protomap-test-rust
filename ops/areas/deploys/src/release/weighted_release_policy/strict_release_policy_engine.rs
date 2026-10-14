use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the strict release policy calculation.
pub trait StrictReleasePolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictReleasePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictReleasePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictReleasePolicyAdjuster for TieredStrictReleasePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictReleasePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictReleasePolicyAdjuster {
    pub cap: f64,
}

impl StrictReleasePolicyAdjuster for CapStrictReleasePolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictReleasePolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleasePolicyTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl StrictReleasePolicyAdjuster for StrictReleasePolicyTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &StrictReleasePolicy, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryStrictReleasePolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictReleasePolicyAdjuster for CategoryStrictReleasePolicyAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &StrictReleasePolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictReleasePolicyAdjuster {
    pub minimum: f64,
}

impl StrictReleasePolicyAdjuster for MinimumStrictReleasePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictReleasePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictReleasePolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictReleasePolicyAdjuster for FlaggedStrictReleasePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictReleasePolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict release policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictReleasePolicyEngine {
    pub steps: Vec<Rc<dyn StrictReleasePolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictReleasePolicyEngine {
    pub fn run(&mut self, item: &StrictReleasePolicy) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictReleasePolicyAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictReleasePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_release_policy_engine(flags: &flags::Registry) -> StrictReleasePolicyEngine {
    StrictReleasePolicyEngine {
        steps: vec![
            Rc::new(TieredStrictReleasePolicyAdjuster {
                bounds: vec![298.0, 1655.0, 1957.0, 6624.0],
                rates: vec![1.03, 0.97, 1.07, 1.1],
            }) as Rc<dyn StrictReleasePolicyAdjuster>,
            Rc::new(CapStrictReleasePolicyAdjuster { cap: 44641.0 }),
            Rc::new(StrictReleasePolicyTtlSecondsOffsetAdjuster { weight: 0.86 }),
            Rc::new(CategoryStrictReleasePolicyAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.78),
                    ("north".to_string(), 0.86),
                    ("south".to_string(), 1.38),
                ]),
            }),
            Rc::new(MinimumStrictReleasePolicyAdjuster { minimum: 34.6 }),
            Rc::new(FlaggedStrictReleasePolicyAdjuster {
                flags: flags.clone(),
                flag: "strict-release-policy-default".to_string(),
                percent: 1.6,
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 12539.2,
    }
}
