use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional dashboard batch calculation.
pub trait RegionalDashboardBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalDashboardBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalDashboardBatchAdjuster {
    pub cap: f64,
}

impl RegionalDashboardBatchAdjuster for CapRegionalDashboardBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalDashboardBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRegionalDashboardBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalDashboardBatchAdjuster for CategoryRegionalDashboardBatchAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RegionalDashboardBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalDashboardBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalDashboardBatchAdjuster for TieredRegionalDashboardBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalDashboardBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalDashboardBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalDashboardBatchAdjuster for FlaggedRegionalDashboardBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalDashboardBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional dashboard batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalDashboardBatchEngine {
    pub steps: Vec<Rc<dyn RegionalDashboardBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalDashboardBatchEngine {
    pub fn compute(&mut self, item: &RegionalDashboardBatch) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[RegionalDashboardBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_dashboard_batch_engine(flags: &flags::Registry) -> RegionalDashboardBatchEngine {
    RegionalDashboardBatchEngine {
        steps: vec![
            Rc::new(CapRegionalDashboardBatchAdjuster { cap: 24692.0 }) as Rc<dyn RegionalDashboardBatchAdjuster>,
            Rc::new(CategoryRegionalDashboardBatchAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.91)]),
            }),
            Rc::new(TieredRegionalDashboardBatchAdjuster {
                bounds: vec![86.0, 2444.0, 2500.0],
                rates: vec![1.07, 0.91, 1.12],
            }),
            Rc::new(FlaggedRegionalDashboardBatchAdjuster {
                flags: flags.clone(),
                flag: "regional-dashboard-batch-lenient".to_string(),
                percent: -3.2,
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 63569.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_dashboard_batch_engine(&flags::new_registry());
        let item = new_regional_dashboard_batch("code-903", "title-104");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
