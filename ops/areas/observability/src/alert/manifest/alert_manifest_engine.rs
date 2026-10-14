use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert manifest calculation.
pub trait AlertManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertManifest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertManifestStep for FlaggedAlertManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameAlertManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertManifestStep for NameAlertManifestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &AlertManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertManifestCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl AlertManifestStep for AlertManifestCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &AlertManifest, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertManifestAdjuster {
    pub percent: f64,
}

impl AlertManifestStep for PercentAlertManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertManifestAdjuster {
    pub step: f64,
}

impl AlertManifestStep for RoundingAlertManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertManifestAdjuster {
    pub minimum: f64,
}

impl AlertManifestStep for MinimumAlertManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the alert manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertManifestEngine {
    pub steps: Vec<Rc<dyn AlertManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertManifestEngine {
    pub fn price(&mut self, item: &AlertManifest) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[AlertManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_alert_manifest_engine(flags: &flags::Registry) -> AlertManifestEngine {
    AlertManifestEngine {
        steps: vec![
            Rc::new(FlaggedAlertManifestAdjuster {
                flags: flags.clone(),
                flag: "alert-manifest-pending".to_string(),
                percent: -8.8,
            }) as Rc<dyn AlertManifestStep>,
            Rc::new(NameAlertManifestAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.1), ("online".to_string(), 1.38)]),
            }),
            Rc::new(AlertManifestCreatedAtOffsetAdjuster { weight: 1.15 }),
            Rc::new(PercentAlertManifestAdjuster { percent: 4.5 }),
            Rc::new(RoundingAlertManifestAdjuster { step: 0.1 }),
            Rc::new(MinimumAlertManifestAdjuster { minimum: 36.6 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 27558.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_manifest_engine(&flags::new_registry());
        let item = new_alert_manifest("code-781", "name-236");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
