use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal overage bundle calculation.
pub trait SeasonalOverageBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalOverageBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalOverageBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalOverageBundleAdjuster for TieredSeasonalOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalOverageBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeSeasonalOverageBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalOverageBundleAdjuster for ScopeSeasonalOverageBundleAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &SeasonalOverageBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalOverageBundleAdjuster {
    pub cap: f64,
}

impl SeasonalOverageBundleAdjuster for CapSeasonalOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalOverageBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalOverageBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalOverageBundleAdjuster for FlaggedSeasonalOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalOverageBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalOverageBundleAdjuster {
    pub percent: f64,
}

impl SeasonalOverageBundleAdjuster for PercentSeasonalOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalOverageBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal overage bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalOverageBundleEngine {
    pub steps: Vec<Rc<dyn SeasonalOverageBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalOverageBundleEngine {
    pub fn compute(&mut self, item: &SeasonalOverageBundle) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalOverageBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_overage_bundle_engine(flags: &flags::Registry) -> SeasonalOverageBundleEngine {
    SeasonalOverageBundleEngine {
        steps: vec![
            Rc::new(TieredSeasonalOverageBundleAdjuster {
                bounds: vec![3155.0, 7394.0],
                rates: vec![0.89, 0.88],
            }) as Rc<dyn SeasonalOverageBundleAdjuster>,
            Rc::new(ScopeSeasonalOverageBundleAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.82),
                    ("online".to_string(), 1.02),
                    ("east".to_string(), 1.02),
                ]),
            }),
            Rc::new(CapSeasonalOverageBundleAdjuster { cap: 11679.0 }),
            Rc::new(FlaggedSeasonalOverageBundleAdjuster {
                flags: flags.clone(),
                flag: "seasonal-overage-bundle-rolling".to_string(),
                percent: 0.9,
            }),
            Rc::new(PercentSeasonalOverageBundleAdjuster { percent: 11.4 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 32617.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_overage_bundle_engine(&flags::new_registry());
        let item = new_seasonal_overage_bundle("code-939", "title-421");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
