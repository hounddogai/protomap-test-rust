use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the strict dashboard digest calculation.
pub trait StrictDashboardDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictDashboardDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictDashboardDigestAdjuster {
    pub percent: f64,
}

impl StrictDashboardDigestAdjuster for PercentStrictDashboardDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictDashboardDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictDashboardDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictDashboardDigestAdjuster for TieredStrictDashboardDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictDashboardDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardDigestFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl StrictDashboardDigestAdjuster for StrictDashboardDigestFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &StrictDashboardDigest, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictDashboardDigestAdjuster {
    pub minimum: f64,
}

impl StrictDashboardDigestAdjuster for MinimumStrictDashboardDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictDashboardDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict dashboard digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictDashboardDigestEngine {
    pub steps: Vec<Rc<dyn StrictDashboardDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictDashboardDigestEngine {
    pub fn run(&mut self, item: &StrictDashboardDigest) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictDashboardDigestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictDashboardDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_dashboard_digest_engine() -> StrictDashboardDigestEngine {
    StrictDashboardDigestEngine {
        steps: vec![
            Rc::new(PercentStrictDashboardDigestAdjuster { percent: -12.3 }) as Rc<dyn StrictDashboardDigestAdjuster>,
            Rc::new(TieredStrictDashboardDigestAdjuster {
                bounds: vec![700.0, 1956.0, 8356.0],
                rates: vec![0.96, 0.97, 1.1],
            }),
            Rc::new(StrictDashboardDigestFloorValueOffsetAdjuster { weight: 1.8 }),
            Rc::new(MinimumStrictDashboardDigestAdjuster { minimum: 7.4 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 98951.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_dashboard_digest_engine();
        let item = new_strict_dashboard_digest("code-694", "title-582");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
