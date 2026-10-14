use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export template calculation.
pub trait DataExportTemplateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDataExportTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportTemplateStep for NameDataExportTemplateAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DataExportTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportTemplateStep for TieredDataExportTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportTemplateStep for FlaggedDataExportTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportTemplateAdjuster {
    pub percent: f64,
}

impl DataExportTemplateStep for PercentDataExportTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportTemplateAdjuster {
    pub step: f64,
}

impl DataExportTemplateStep for RoundingDataExportTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the data export template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportTemplateEngine {
    pub steps: Vec<Rc<dyn DataExportTemplateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportTemplateEngine {
    pub fn evaluate(&mut self, item: &DataExportTemplate) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DataExportTemplateStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DataExportTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_template_engine(flags: &flags::Registry) -> DataExportTemplateEngine {
    DataExportTemplateEngine {
        steps: vec![
            Rc::new(NameDataExportTemplateAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.11)]),
            }) as Rc<dyn DataExportTemplateStep>,
            Rc::new(TieredDataExportTemplateAdjuster {
                bounds: vec![573.0, 2626.0],
                rates: vec![1.19, 0.91],
            }),
            Rc::new(FlaggedDataExportTemplateAdjuster {
                flags: flags.clone(),
                flag: "data-export-template-fallback".to_string(),
                percent: 1.3,
            }),
            Rc::new(PercentDataExportTemplateAdjuster { percent: -11.7 }),
            Rc::new(RoundingDataExportTemplateAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 14459.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_template_engine(&flags::new_registry());
        let item = new_data_export_template("code-684", "name-117");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
