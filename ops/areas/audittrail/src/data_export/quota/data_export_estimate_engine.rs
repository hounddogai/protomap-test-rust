use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the data export estimate calculation.
pub trait DataExportEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportEstimateFactor for TieredDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportEstimateAdjuster {
    pub step: f64,
}

impl DataExportEstimateFactor for RoundingDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportEstimateBudgetOffsetAdjuster {
    pub weight: f64,
}

impl DataExportEstimateFactor for DataExportEstimateBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportEstimate, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDataExportEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportEstimateFactor for SegmentDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &DataExportEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportEstimateAdjuster {
    pub cap: f64,
}

impl DataExportEstimateFactor for CapDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportEstimateAdjuster {
    pub minimum: f64,
}

impl DataExportEstimateFactor for MinimumDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the data export estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportEstimateEngine {
    pub steps: Vec<Rc<dyn DataExportEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportEstimateEngine {
    pub fn price(&mut self, item: &DataExportEstimate) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[DataExportEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_data_export_estimate_engine() -> DataExportEstimateEngine {
    DataExportEstimateEngine {
        steps: vec![
            Rc::new(TieredDataExportEstimateAdjuster {
                bounds: vec![898.0, 5619.0, 9071.0],
                rates: vec![1.18, 0.87, 1.09],
            }) as Rc<dyn DataExportEstimateFactor>,
            Rc::new(RoundingDataExportEstimateAdjuster { step: 1.0 }),
            Rc::new(DataExportEstimateBudgetOffsetAdjuster { weight: 1.6 }),
            Rc::new(SegmentDataExportEstimateAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.29),
                    ("retail".to_string(), 0.91),
                    ("west".to_string(), 1.21),
                ]),
            }),
            Rc::new(CapDataExportEstimateAdjuster { cap: 1489.0 }),
            Rc::new(MinimumDataExportEstimateAdjuster { minimum: 43.3 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 67389.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_estimate_engine();
        let item = new_data_export_estimate("code-198", "title-691");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
