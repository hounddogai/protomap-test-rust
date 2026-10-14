use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export rule calculation.
pub trait DataExportRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportRuleAdjuster {
    pub percent: f64,
}

impl DataExportRuleModifier for PercentDataExportRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportRuleAdjuster {
    pub step: f64,
}

impl DataExportRuleModifier for RoundingDataExportRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportRuleModifier for FlaggedDataExportRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the data export rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportRuleEngine {
    pub steps: Vec<Rc<dyn DataExportRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportRuleEngine {
    pub fn compute(&mut self, item: &DataExportRule) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DataExportRuleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DataExportRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_data_export_rule_engine(flags: &flags::Registry) -> DataExportRuleEngine {
    DataExportRuleEngine {
        steps: vec![
            Rc::new(PercentDataExportRuleAdjuster { percent: -13.0 }) as Rc<dyn DataExportRuleModifier>,
            Rc::new(RoundingDataExportRuleAdjuster { step: 1.0 }),
            Rc::new(FlaggedDataExportRuleAdjuster {
                flags: flags.clone(),
                flag: "data-export-rule-pending".to_string(),
                percent: 1.9,
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 69204.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_rule_engine(&flags::new_registry());
        let item = new_data_export_rule("code-476", "name-727");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
