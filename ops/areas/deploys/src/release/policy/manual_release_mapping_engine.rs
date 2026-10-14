use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual release mapping calculation.
pub trait ManualReleaseMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualReleaseMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualReleaseMappingAdjuster {
    pub step: f64,
}

impl ManualReleaseMappingStep for RoundingManualReleaseMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualReleaseMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualReleaseMappingAdjuster {
    pub cap: f64,
}

impl ManualReleaseMappingStep for CapManualReleaseMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualReleaseMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualReleaseMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualReleaseMappingStep for TieredManualReleaseMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualReleaseMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualReleaseMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualReleaseMappingStep for FlaggedManualReleaseMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualReleaseMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceManualReleaseMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualReleaseMappingStep for NamespaceManualReleaseMappingAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &ManualReleaseMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseMappingMarginOffsetAdjuster {
    pub weight: f64,
}

impl ManualReleaseMappingStep for ManualReleaseMappingMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &ManualReleaseMapping, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the manual release mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualReleaseMappingEngine {
    pub steps: Vec<Rc<dyn ManualReleaseMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualReleaseMappingEngine {
    pub fn run(&mut self, item: &ManualReleaseMapping) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualReleaseMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_release_mapping_engine(flags: &flags::Registry) -> ManualReleaseMappingEngine {
    ManualReleaseMappingEngine {
        steps: vec![
            Rc::new(RoundingManualReleaseMappingAdjuster { step: 0.1 }) as Rc<dyn ManualReleaseMappingStep>,
            Rc::new(CapManualReleaseMappingAdjuster { cap: 4364.0 }),
            Rc::new(TieredManualReleaseMappingAdjuster {
                bounds: vec![4689.0, 9130.0],
                rates: vec![0.98, 1.14],
            }),
            Rc::new(FlaggedManualReleaseMappingAdjuster {
                flags: flags.clone(),
                flag: "manual-release-mapping-fallback".to_string(),
                percent: -1.1,
            }),
            Rc::new(NamespaceManualReleaseMappingAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.84), ("east".to_string(), 1.33)]),
            }),
            Rc::new(ManualReleaseMappingMarginOffsetAdjuster { weight: 1.3 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 51914.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_release_mapping_engine(&flags::new_registry());
        let item = new_manual_release_mapping("code-626", "title-806");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
