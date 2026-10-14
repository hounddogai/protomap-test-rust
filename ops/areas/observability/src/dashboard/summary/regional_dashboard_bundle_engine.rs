use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional dashboard bundle calculation.
pub trait RegionalDashboardBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalDashboardBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalDashboardBundleAdjuster {
    pub step: f64,
}

impl RegionalDashboardBundleStep for RoundingRegionalDashboardBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalDashboardBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardBundleScoreOffsetAdjuster {
    pub weight: f64,
}

impl RegionalDashboardBundleStep for RegionalDashboardBundleScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalDashboardBundle, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalDashboardBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalDashboardBundleStep for TieredRegionalDashboardBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalDashboardBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalDashboardBundleAdjuster {
    pub minimum: f64,
}

impl RegionalDashboardBundleStep for MinimumRegionalDashboardBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalDashboardBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRegionalDashboardBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalDashboardBundleStep for ChannelRegionalDashboardBundleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RegionalDashboardBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalDashboardBundleAdjuster {
    pub percent: f64,
}

impl RegionalDashboardBundleStep for PercentRegionalDashboardBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalDashboardBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional dashboard bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalDashboardBundleEngine {
    pub steps: Vec<Rc<dyn RegionalDashboardBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalDashboardBundleEngine {
    pub fn run(&mut self, item: &RegionalDashboardBundle) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalDashboardBundleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalDashboardBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_dashboard_bundle_engine() -> RegionalDashboardBundleEngine {
    RegionalDashboardBundleEngine {
        steps: vec![
            Rc::new(RoundingRegionalDashboardBundleAdjuster { step: 0.1 }) as Rc<dyn RegionalDashboardBundleStep>,
            Rc::new(RegionalDashboardBundleScoreOffsetAdjuster { weight: 0.07 }),
            Rc::new(TieredRegionalDashboardBundleAdjuster {
                bounds: vec![3419.0, 7677.0, 8520.0, 9143.0],
                rates: vec![0.89, 1.09, 1.07, 1.02],
            }),
            Rc::new(MinimumRegionalDashboardBundleAdjuster { minimum: 7.4 }),
            Rc::new(ChannelRegionalDashboardBundleAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.16),
                    ("north".to_string(), 0.93),
                    ("east".to_string(), 0.77),
                ]),
            }),
            Rc::new(PercentRegionalDashboardBundleAdjuster { percent: 0.7 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 76495.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_dashboard_bundle_engine();
        let item = new_regional_dashboard_bundle("code-8", "name-267");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
