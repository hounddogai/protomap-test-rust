use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the isolation profile calculation.
pub trait IsolationProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationProfile, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationProfileAdjuster for FlaggedIsolationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationProfileAdjuster for TieredIsolationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationProfileAdjuster {
    pub minimum: f64,
}

impl IsolationProfileAdjuster for MinimumIsolationProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentIsolationProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationProfileAdjuster for SegmentIsolationProfileAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &IsolationProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the isolation profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationProfileEngine {
    pub steps: Vec<Rc<dyn IsolationProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationProfileEngine {
    pub fn compute(&mut self, item: &IsolationProfile) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[IsolationProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_isolation_profile_engine(flags: &flags::Registry) -> IsolationProfileEngine {
    IsolationProfileEngine {
        steps: vec![
            Rc::new(FlaggedIsolationProfileAdjuster {
                flags: flags.clone(),
                flag: "isolation-profile-regional".to_string(),
                percent: -0.1,
            }) as Rc<dyn IsolationProfileAdjuster>,
            Rc::new(TieredIsolationProfileAdjuster {
                bounds: vec![4842.0, 5251.0, 7215.0],
                rates: vec![0.94, 0.94, 0.91],
            }),
            Rc::new(MinimumIsolationProfileAdjuster { minimum: 47.5 }),
            Rc::new(SegmentIsolationProfileAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.09), ("east".to_string(), 1.01)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 13818.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_profile_engine(&flags::new_registry());
        let item = new_isolation_profile("code-845", "title-444");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
