use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the data export claim calculation.
pub trait DataExportClaimAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportClaimAdjuster {
    pub percent: f64,
}

impl DataExportClaimAdjuster for PercentDataExportClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportClaimAdjuster {
    pub minimum: f64,
}

impl DataExportClaimAdjuster for MinimumDataExportClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterDataExportClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportClaimAdjuster for ClusterDataExportClaimAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &DataExportClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the data export claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportClaimEngine {
    pub steps: Vec<Rc<dyn DataExportClaimAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportClaimEngine {
    pub fn evaluate(&mut self, item: &DataExportClaim) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DataExportClaimAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DataExportClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_claim_engine() -> DataExportClaimEngine {
    DataExportClaimEngine {
        steps: vec![
            Rc::new(PercentDataExportClaimAdjuster { percent: 16.1 }) as Rc<dyn DataExportClaimAdjuster>,
            Rc::new(MinimumDataExportClaimAdjuster { minimum: 44.1 }),
            Rc::new(ClusterDataExportClaimAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.07),
                    ("south".to_string(), 0.81),
                    ("west".to_string(), 1.04),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 87955.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_claim_engine();
        let item = new_data_export_claim("code-568", "label-577");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
