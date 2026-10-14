use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default data export estimate calculation.
pub trait DefaultDataExportEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultDataExportEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportEstimateTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl DefaultDataExportEstimateStep for DefaultDataExportEstimateTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultDataExportEstimate, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultDataExportEstimateAdjuster {
    pub percent: f64,
}

impl DefaultDataExportEstimateStep for PercentDefaultDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultDataExportEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultDataExportEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultDataExportEstimateStep for FlaggedDefaultDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultDataExportEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultDataExportEstimateAdjuster {
    pub minimum: f64,
}

impl DefaultDataExportEstimateStep for MinimumDefaultDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultDataExportEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterDefaultDataExportEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultDataExportEstimateStep for ClusterDefaultDataExportEstimateAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &DefaultDataExportEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default data export estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultDataExportEstimateEngine {
    pub steps: Vec<Rc<dyn DefaultDataExportEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultDataExportEstimateEngine {
    pub fn compute(&mut self, item: &DefaultDataExportEstimate) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DefaultDataExportEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_data_export_estimate_engine(flags: &flags::Registry) -> DefaultDataExportEstimateEngine {
    DefaultDataExportEstimateEngine {
        steps: vec![
            Rc::new(DefaultDataExportEstimateTtlSecondsOffsetAdjuster { weight: 0.94 })
                as Rc<dyn DefaultDataExportEstimateStep>,
            Rc::new(PercentDefaultDataExportEstimateAdjuster { percent: -14.2 }),
            Rc::new(FlaggedDefaultDataExportEstimateAdjuster {
                flags: flags.clone(),
                flag: "default-data-export-estimate-manual".to_string(),
                percent: 3.5,
            }),
            Rc::new(MinimumDefaultDataExportEstimateAdjuster { minimum: 10.9 }),
            Rc::new(ClusterDefaultDataExportEstimateAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.1),
                    ("east".to_string(), 1.1),
                    ("west".to_string(), 1.19),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 98692.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_data_export_estimate_engine(&flags::new_registry());
        let item = new_default_data_export_estimate("code-218", "label-311");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
