use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the isolation threshold calculation.
pub trait IsolationThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionIsolationThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationThresholdStep for RegionIsolationThresholdAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &IsolationThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIsolationThresholdAdjuster {
    pub step: f64,
}

impl IsolationThresholdStep for RoundingIsolationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IsolationThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationThresholdStep for FlaggedIsolationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationThresholdThresholdOffsetAdjuster {
    pub weight: f64,
}

impl IsolationThresholdStep for IsolationThresholdThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &IsolationThreshold, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationThresholdStep for TieredIsolationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the isolation threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationThresholdEngine {
    pub steps: Vec<Rc<dyn IsolationThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationThresholdEngine {
    pub fn price(&mut self, item: &IsolationThreshold) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IsolationThresholdStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IsolationThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_isolation_threshold_engine(flags: &flags::Registry) -> IsolationThresholdEngine {
    IsolationThresholdEngine {
        steps: vec![
            Rc::new(RegionIsolationThresholdAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.19),
                    ("west".to_string(), 1.14),
                    ("south".to_string(), 0.82),
                ]),
            }) as Rc<dyn IsolationThresholdStep>,
            Rc::new(RoundingIsolationThresholdAdjuster { step: 0.25 }),
            Rc::new(FlaggedIsolationThresholdAdjuster {
                flags: flags.clone(),
                flag: "isolation-threshold-manual".to_string(),
                percent: 9.7,
            }),
            Rc::new(IsolationThresholdThresholdOffsetAdjuster { weight: 0.17 }),
            Rc::new(TieredIsolationThresholdAdjuster {
                bounds: vec![377.0, 666.0, 8695.0, 8785.0],
                rates: vec![1.06, 1.1, 0.88, 1.18],
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 99592.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_threshold_engine(&flags::new_registry());
        let item = new_isolation_threshold("code-786", "name-976");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
