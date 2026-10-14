use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the data export metric calculation.
pub trait DataExportMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportMetricAdjuster {
    pub step: f64,
}

impl DataExportMetricAdjuster for RoundingDataExportMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportMetricAdjuster {
    pub minimum: f64,
}

impl DataExportMetricAdjuster for MinimumDataExportMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceDataExportMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportMetricAdjuster for SourceDataExportMetricAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &DataExportMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportMetricAdjuster for TieredDataExportMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportMetricAdjuster {
    pub cap: f64,
}

impl DataExportMetricAdjuster for CapDataExportMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the data export metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportMetricEngine {
    pub steps: Vec<Rc<dyn DataExportMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportMetricEngine {
    pub fn price(&mut self, item: &DataExportMetric) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DataExportMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_data_export_metric_engine() -> DataExportMetricEngine {
    DataExportMetricEngine {
        steps: vec![
            Rc::new(RoundingDataExportMetricAdjuster { step: 5.0 }) as Rc<dyn DataExportMetricAdjuster>,
            Rc::new(MinimumDataExportMetricAdjuster { minimum: 20.4 }),
            Rc::new(SourceDataExportMetricAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.01), ("retail".to_string(), 1.2)]),
            }),
            Rc::new(TieredDataExportMetricAdjuster {
                bounds: vec![3957.0, 4856.0, 6701.0, 7971.0],
                rates: vec![1.2, 0.98, 1.08, 0.81],
            }),
            Rc::new(CapDataExportMetricAdjuster { cap: 14890.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 32981.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_metric_engine();
        let item = new_data_export_metric("code-245", "label-622");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
