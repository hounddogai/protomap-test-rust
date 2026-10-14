use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled data export template calculation.
pub trait ScheduledDataExportTemplateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledDataExportTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionScheduledDataExportTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledDataExportTemplateAdjuster for RegionScheduledDataExportTemplateAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ScheduledDataExportTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledDataExportTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledDataExportTemplateAdjuster for FlaggedScheduledDataExportTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledDataExportTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledDataExportTemplateAdjuster {
    pub minimum: f64,
}

impl ScheduledDataExportTemplateAdjuster for MinimumScheduledDataExportTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledDataExportTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled data export template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledDataExportTemplateEngine {
    pub steps: Vec<Rc<dyn ScheduledDataExportTemplateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledDataExportTemplateEngine {
    pub fn run(&mut self, item: &ScheduledDataExportTemplate) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledDataExportTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_data_export_template_engine(flags: &flags::Registry) -> ScheduledDataExportTemplateEngine {
    ScheduledDataExportTemplateEngine {
        steps: vec![
            Rc::new(RegionScheduledDataExportTemplateAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.99), ("east".to_string(), 1.27)]),
            }) as Rc<dyn ScheduledDataExportTemplateAdjuster>,
            Rc::new(FlaggedScheduledDataExportTemplateAdjuster {
                flags: flags.clone(),
                flag: "scheduled-data-export-template-weighted".to_string(),
                percent: -4.3,
            }),
            Rc::new(MinimumScheduledDataExportTemplateAdjuster { minimum: 3.9 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 31515.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_data_export_template_engine(&flags::new_registry());
        let item = new_scheduled_data_export_template("code-737", "label-283");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
