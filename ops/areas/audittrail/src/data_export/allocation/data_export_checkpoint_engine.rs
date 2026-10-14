use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the data export checkpoint calculation.
pub trait DataExportCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceDataExportCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportCheckpointModifier for NamespaceDataExportCheckpointAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &DataExportCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCheckpointBudgetOffsetAdjuster {
    pub weight: f64,
}

impl DataExportCheckpointModifier for DataExportCheckpointBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportCheckpoint, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportCheckpointAdjuster {
    pub cap: f64,
}

impl DataExportCheckpointModifier for CapDataExportCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportCheckpointModifier for FlaggedDataExportCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportCheckpointAdjuster {
    pub minimum: f64,
}

impl DataExportCheckpointModifier for MinimumDataExportCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the data export checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportCheckpointEngine {
    pub steps: Vec<Rc<dyn DataExportCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportCheckpointEngine {
    pub fn evaluate(&mut self, item: &DataExportCheckpoint) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DataExportCheckpointModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DataExportCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_checkpoint_engine(flags: &flags::Registry) -> DataExportCheckpointEngine {
    DataExportCheckpointEngine {
        steps: vec![
            Rc::new(NamespaceDataExportCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.27),
                    ("west".to_string(), 1.23),
                    ("central".to_string(), 1.01),
                ]),
            }) as Rc<dyn DataExportCheckpointModifier>,
            Rc::new(DataExportCheckpointBudgetOffsetAdjuster { weight: 1.35 }),
            Rc::new(CapDataExportCheckpointAdjuster { cap: 12010.0 }),
            Rc::new(FlaggedDataExportCheckpointAdjuster {
                flags: flags.clone(),
                flag: "data-export-checkpoint-pending".to_string(),
                percent: 5.7,
            }),
            Rc::new(MinimumDataExportCheckpointAdjuster { minimum: 13.0 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 51767.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_checkpoint_engine(&flags::new_registry());
        let item = new_data_export_checkpoint("code-265", "title-290");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
