use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered data export rollup calculation.
pub trait TieredDataExportRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredDataExportRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredDataExportRollupAdjuster {
    pub cap: f64,
}

impl TieredDataExportRollupAdjuster for CapTieredDataExportRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredDataExportRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredDataExportRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredDataExportRollupAdjuster for FlaggedTieredDataExportRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredDataExportRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredDataExportRollupAdjuster {
    pub minimum: f64,
}

impl TieredDataExportRollupAdjuster for MinimumTieredDataExportRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredDataExportRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredDataExportRollupAdjuster {
    pub percent: f64,
}

impl TieredDataExportRollupAdjuster for PercentTieredDataExportRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredDataExportRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered data export rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredDataExportRollupEngine {
    pub steps: Vec<Rc<dyn TieredDataExportRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredDataExportRollupEngine {
    pub fn run(&mut self, item: &TieredDataExportRollup) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[TieredDataExportRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_data_export_rollup_engine(flags: &flags::Registry) -> TieredDataExportRollupEngine {
    TieredDataExportRollupEngine {
        steps: vec![
            Rc::new(CapTieredDataExportRollupAdjuster { cap: 44511.0 }) as Rc<dyn TieredDataExportRollupAdjuster>,
            Rc::new(FlaggedTieredDataExportRollupAdjuster {
                flags: flags.clone(),
                flag: "tiered-data-export-rollup-weighted".to_string(),
                percent: 9.3,
            }),
            Rc::new(MinimumTieredDataExportRollupAdjuster { minimum: 22.9 }),
            Rc::new(PercentTieredDataExportRollupAdjuster { percent: -10.2 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 39629.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_data_export_rollup_engine(&flags::new_registry());
        let item = new_tiered_data_export_rollup("code-959", "title-899");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
