use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy redaction rollup calculation.
pub trait LegacyRedactionRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRedactionRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLegacyRedactionRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyRedactionRollupAdjuster for NameLegacyRedactionRollupAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LegacyRedactionRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyRedactionRollupAdjuster {
    pub percent: f64,
}

impl LegacyRedactionRollupAdjuster for PercentLegacyRedactionRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyRedactionRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyRedactionRollupAdjuster {
    pub minimum: f64,
}

impl LegacyRedactionRollupAdjuster for MinimumLegacyRedactionRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyRedactionRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionRollupQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl LegacyRedactionRollupAdjuster for LegacyRedactionRollupQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyRedactionRollup, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyRedactionRollupAdjuster {
    pub cap: f64,
}

impl LegacyRedactionRollupAdjuster for CapLegacyRedactionRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyRedactionRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the legacy redaction rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRedactionRollupEngine {
    pub steps: Vec<Rc<dyn LegacyRedactionRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRedactionRollupEngine {
    pub fn run(&mut self, item: &LegacyRedactionRollup) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyRedactionRollupAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyRedactionRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_redaction_rollup_engine() -> LegacyRedactionRollupEngine {
    LegacyRedactionRollupEngine {
        steps: vec![
            Rc::new(NameLegacyRedactionRollupAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.9)]),
            }) as Rc<dyn LegacyRedactionRollupAdjuster>,
            Rc::new(PercentLegacyRedactionRollupAdjuster { percent: -1.3 }),
            Rc::new(MinimumLegacyRedactionRollupAdjuster { minimum: 43.0 }),
            Rc::new(LegacyRedactionRollupQuotaUnitsOffsetAdjuster { weight: 0.45 }),
            Rc::new(CapLegacyRedactionRollupAdjuster { cap: 16455.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 70507.1,
    }
}
