use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default alert claim calculation.
pub trait DefaultAlertClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultAlertClaim, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultAlertClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultAlertClaimFactor for FlaggedDefaultAlertClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultAlertClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertClaimQuantityOffsetAdjuster {
    pub weight: f64,
}

impl DefaultAlertClaimFactor for DefaultAlertClaimQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultAlertClaim, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultAlertClaimAdjuster {
    pub percent: f64,
}

impl DefaultAlertClaimFactor for PercentDefaultAlertClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultAlertClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default alert claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultAlertClaimEngine {
    pub steps: Vec<Rc<dyn DefaultAlertClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultAlertClaimEngine {
    pub fn run(&mut self, item: &DefaultAlertClaim) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DefaultAlertClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_alert_claim_engine(flags: &flags::Registry) -> DefaultAlertClaimEngine {
    DefaultAlertClaimEngine {
        steps: vec![
            Rc::new(FlaggedDefaultAlertClaimAdjuster {
                flags: flags.clone(),
                flag: "default-alert-claim-scheduled".to_string(),
                percent: 6.3,
            }) as Rc<dyn DefaultAlertClaimFactor>,
            Rc::new(DefaultAlertClaimQuantityOffsetAdjuster { weight: 0.97 }),
            Rc::new(PercentDefaultAlertClaimAdjuster { percent: 20.3 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 39687.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_alert_claim_engine(&flags::new_registry());
        let item = new_default_alert_claim("code-941", "label-910");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
