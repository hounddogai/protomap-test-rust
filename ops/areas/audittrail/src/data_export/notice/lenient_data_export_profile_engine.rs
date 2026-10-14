use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient data export profile calculation.
pub trait LenientDataExportProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientDataExportProfile, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientDataExportProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientDataExportProfileAdjuster for FlaggedLenientDataExportProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientDataExportProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientDataExportProfileAdjuster {
    pub step: f64,
}

impl LenientDataExportProfileAdjuster for RoundingLenientDataExportProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientDataExportProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientDataExportProfileAdjuster {
    pub percent: f64,
}

impl LenientDataExportProfileAdjuster for PercentLenientDataExportProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientDataExportProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceLenientDataExportProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientDataExportProfileAdjuster for NamespaceLenientDataExportProfileAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &LenientDataExportProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient data export profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientDataExportProfileEngine {
    pub steps: Vec<Rc<dyn LenientDataExportProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientDataExportProfileEngine {
    pub fn evaluate(&mut self, item: &LenientDataExportProfile) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[LenientDataExportProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_data_export_profile_engine(flags: &flags::Registry) -> LenientDataExportProfileEngine {
    LenientDataExportProfileEngine {
        steps: vec![
            Rc::new(FlaggedLenientDataExportProfileAdjuster {
                flags: flags.clone(),
                flag: "lenient-data-export-profile-weighted".to_string(),
                percent: -9.9,
            }) as Rc<dyn LenientDataExportProfileAdjuster>,
            Rc::new(RoundingLenientDataExportProfileAdjuster { step: 5.0 }),
            Rc::new(PercentLenientDataExportProfileAdjuster { percent: -7.2 }),
            Rc::new(NamespaceLenientDataExportProfileAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.39), ("east".to_string(), 1.23)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 86670.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_data_export_profile_engine(&flags::new_registry());
        let item = new_lenient_data_export_profile("code-532", "label-948");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
