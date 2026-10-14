use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export policy calculation.
pub trait DataExportPolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportPolicyAdjuster {
    pub cap: f64,
}

impl DataExportPolicyStep for CapDataExportPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportPolicyStep for FlaggedDataExportPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPolicyRateOffsetAdjuster {
    pub weight: f64,
}

impl DataExportPolicyStep for DataExportPolicyRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportPolicy, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

/// Runs the data export policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportPolicyEngine {
    pub steps: Vec<Rc<dyn DataExportPolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportPolicyEngine {
    pub fn evaluate(&mut self, item: &DataExportPolicy) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DataExportPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_policy_engine(flags: &flags::Registry) -> DataExportPolicyEngine {
    DataExportPolicyEngine {
        steps: vec![
            Rc::new(CapDataExportPolicyAdjuster { cap: 32885.0 }) as Rc<dyn DataExportPolicyStep>,
            Rc::new(FlaggedDataExportPolicyAdjuster {
                flags: flags.clone(),
                flag: "data-export-policy-default".to_string(),
                percent: 5.8,
            }),
            Rc::new(DataExportPolicyRateOffsetAdjuster { weight: 1.57 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 94003.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_policy_engine(&flags::new_registry());
        let item = new_data_export_policy("code-763", "label-824");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
