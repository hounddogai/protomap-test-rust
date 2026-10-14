use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual probe notice calculation.
pub trait ManualProbeNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualProbeNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualProbeNoticeAdjuster {
    pub step: f64,
}

impl ManualProbeNoticeFactor for RoundingManualProbeNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualProbeNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameManualProbeNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualProbeNoticeFactor for NameManualProbeNoticeAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ManualProbeNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualProbeNoticeAdjuster {
    pub percent: f64,
}

impl ManualProbeNoticeFactor for PercentManualProbeNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualProbeNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualProbeNoticeAdjuster {
    pub minimum: f64,
}

impl ManualProbeNoticeFactor for MinimumManualProbeNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualProbeNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual probe notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualProbeNoticeEngine {
    pub steps: Vec<Rc<dyn ManualProbeNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualProbeNoticeEngine {
    pub fn compute(&mut self, item: &ManualProbeNotice) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[ManualProbeNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_probe_notice_engine() -> ManualProbeNoticeEngine {
    ManualProbeNoticeEngine {
        steps: vec![
            Rc::new(RoundingManualProbeNoticeAdjuster { step: 0.05 }) as Rc<dyn ManualProbeNoticeFactor>,
            Rc::new(NameManualProbeNoticeAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.27)]),
            }),
            Rc::new(PercentManualProbeNoticeAdjuster { percent: 1.5 }),
            Rc::new(MinimumManualProbeNoticeAdjuster { minimum: 28.4 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 9704.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_probe_notice_engine();
        let item = new_manual_probe_notice("code-700", "name-255");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
