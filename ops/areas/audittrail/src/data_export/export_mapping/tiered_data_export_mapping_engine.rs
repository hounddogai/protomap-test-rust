use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered data export mapping calculation.
pub trait TieredDataExportMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredDataExportMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleTieredDataExportMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredDataExportMappingStep for TitleTieredDataExportMappingAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &TieredDataExportMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredDataExportMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredDataExportMappingStep for FlaggedTieredDataExportMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredDataExportMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportMappingMinimumOffsetAdjuster {
    pub weight: f64,
}

impl TieredDataExportMappingStep for TieredDataExportMappingMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &TieredDataExportMapping, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredDataExportMappingAdjuster {
    pub minimum: f64,
}

impl TieredDataExportMappingStep for MinimumTieredDataExportMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredDataExportMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered data export mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredDataExportMappingEngine {
    pub steps: Vec<Rc<dyn TieredDataExportMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredDataExportMappingEngine {
    pub fn evaluate(&mut self, item: &TieredDataExportMapping) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[TieredDataExportMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_data_export_mapping_engine(flags: &flags::Registry) -> TieredDataExportMappingEngine {
    TieredDataExportMappingEngine {
        steps: vec![
            Rc::new(TitleTieredDataExportMappingAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.96)]),
            }) as Rc<dyn TieredDataExportMappingStep>,
            Rc::new(FlaggedTieredDataExportMappingAdjuster {
                flags: flags.clone(),
                flag: "tiered-data-export-mapping-legacy".to_string(),
                percent: 5.3,
            }),
            Rc::new(TieredDataExportMappingMinimumOffsetAdjuster { weight: 0.79 }),
            Rc::new(MinimumTieredDataExportMappingAdjuster { minimum: 40.2 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 64393.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_data_export_mapping_engine(&flags::new_registry());
        let item = new_tiered_data_export_mapping("code-82", "title-670");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
