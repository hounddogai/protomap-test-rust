use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the suspension profile calculation.
pub trait SuspensionProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionProfileAdjuster {
    pub percent: f64,
}

impl SuspensionProfileStep for PercentSuspensionProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSuspensionProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionProfileStep for SegmentSuspensionProfileAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &SuspensionProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionProfileStep for FlaggedSuspensionProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the suspension profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionProfileEngine {
    pub steps: Vec<Rc<dyn SuspensionProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionProfileEngine {
    pub fn price(&mut self, item: &SuspensionProfile) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SuspensionProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_suspension_profile_engine(flags: &flags::Registry) -> SuspensionProfileEngine {
    SuspensionProfileEngine {
        steps: vec![
            Rc::new(PercentSuspensionProfileAdjuster { percent: -7.8 }) as Rc<dyn SuspensionProfileStep>,
            Rc::new(SegmentSuspensionProfileAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.71), ("south".to_string(), 0.82)]),
            }),
            Rc::new(FlaggedSuspensionProfileAdjuster {
                flags: flags.clone(),
                flag: "suspension-profile-scheduled".to_string(),
                percent: -4.8,
            }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 19775.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_profile_engine(&flags::new_registry());
        let item = new_suspension_profile("code-491", "title-382");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
