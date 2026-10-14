use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert claim calculation.
pub trait AlertClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertClaimAdjuster {
    pub step: f64,
}

impl AlertClaimStep for RoundingAlertClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleAlertClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertClaimStep for LocaleAlertClaimAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &AlertClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertClaimAdjuster {
    pub minimum: f64,
}

impl AlertClaimStep for MinimumAlertClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertClaimStep for FlaggedAlertClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the alert claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertClaimEngine {
    pub steps: Vec<Rc<dyn AlertClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertClaimEngine {
    pub fn compute(&mut self, item: &AlertClaim) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AlertClaimStep> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[AlertClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_alert_claim_engine(flags: &flags::Registry) -> AlertClaimEngine {
    AlertClaimEngine {
        steps: vec![
            Rc::new(RoundingAlertClaimAdjuster { step: 0.05 }) as Rc<dyn AlertClaimStep>,
            Rc::new(LocaleAlertClaimAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.22),
                    ("north".to_string(), 1.21),
                    ("central".to_string(), 1.26),
                ]),
            }),
            Rc::new(MinimumAlertClaimAdjuster { minimum: 4.9 }),
            Rc::new(FlaggedAlertClaimAdjuster {
                flags: flags.clone(),
                flag: "alert-claim-weighted".to_string(),
                percent: -8.5,
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 66731.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_claim_engine(&flags::new_registry());
        let item = new_alert_claim("code-62", "label-811");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
