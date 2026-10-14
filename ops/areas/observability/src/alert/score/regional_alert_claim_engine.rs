use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional alert claim calculation.
pub trait RegionalAlertClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalAlertClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalAlertClaimAdjuster {
    pub percent: f64,
}

impl RegionalAlertClaimFactor for PercentRegionalAlertClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalAlertClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalAlertClaimAdjuster {
    pub minimum: f64,
}

impl RegionalAlertClaimFactor for MinimumRegionalAlertClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalAlertClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalAlertClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalAlertClaimFactor for FlaggedRegionalAlertClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalAlertClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRegionalAlertClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalAlertClaimFactor for SourceRegionalAlertClaimAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &RegionalAlertClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the regional alert claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalAlertClaimEngine {
    pub steps: Vec<Rc<dyn RegionalAlertClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalAlertClaimEngine {
    pub fn price(&mut self, item: &RegionalAlertClaim) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RegionalAlertClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_alert_claim_engine(flags: &flags::Registry) -> RegionalAlertClaimEngine {
    RegionalAlertClaimEngine {
        steps: vec![
            Rc::new(PercentRegionalAlertClaimAdjuster { percent: 3.6 }) as Rc<dyn RegionalAlertClaimFactor>,
            Rc::new(MinimumRegionalAlertClaimAdjuster { minimum: 39.4 }),
            Rc::new(FlaggedRegionalAlertClaimAdjuster {
                flags: flags.clone(),
                flag: "regional-alert-claim-strict".to_string(),
                percent: -3.2,
            }),
            Rc::new(SourceRegionalAlertClaimAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.36), ("central".to_string(), 0.79)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 98192.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_alert_claim_engine(&flags::new_registry());
        let item = new_regional_alert_claim("code-623", "title-901");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
