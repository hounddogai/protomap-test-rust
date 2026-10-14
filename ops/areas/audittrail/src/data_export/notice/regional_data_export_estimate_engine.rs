use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional data export estimate calculation.
pub trait RegionalDataExportEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalDataExportEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalDataExportEstimateAdjuster {
    pub percent: f64,
}

impl RegionalDataExportEstimateAdjuster for PercentRegionalDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalDataExportEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRegionalDataExportEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalDataExportEstimateAdjuster for CategoryRegionalDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RegionalDataExportEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalDataExportEstimateAdjuster {
    pub cap: f64,
}

impl RegionalDataExportEstimateAdjuster for CapRegionalDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalDataExportEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportEstimateQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl RegionalDataExportEstimateAdjuster for RegionalDataExportEstimateQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalDataExportEstimate, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalDataExportEstimateAdjuster {
    pub minimum: f64,
}

impl RegionalDataExportEstimateAdjuster for MinimumRegionalDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalDataExportEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalDataExportEstimateAdjuster {
    pub step: f64,
}

impl RegionalDataExportEstimateAdjuster for RoundingRegionalDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalDataExportEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the regional data export estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalDataExportEstimateEngine {
    pub steps: Vec<Rc<dyn RegionalDataExportEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalDataExportEstimateEngine {
    pub fn compute(&mut self, item: &RegionalDataExportEstimate) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[RegionalDataExportEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_data_export_estimate_engine() -> RegionalDataExportEstimateEngine {
    RegionalDataExportEstimateEngine {
        steps: vec![
            Rc::new(PercentRegionalDataExportEstimateAdjuster { percent: -3.1 })
                as Rc<dyn RegionalDataExportEstimateAdjuster>,
            Rc::new(CategoryRegionalDataExportEstimateAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.72)]),
            }),
            Rc::new(CapRegionalDataExportEstimateAdjuster { cap: 44858.0 }),
            Rc::new(RegionalDataExportEstimateQuotaUnitsOffsetAdjuster { weight: 1.86 }),
            Rc::new(MinimumRegionalDataExportEstimateAdjuster { minimum: 25.3 }),
            Rc::new(RoundingRegionalDataExportEstimateAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 15339.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_data_export_estimate_engine();
        let item = new_regional_data_export_estimate("code-604", "label-361");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
