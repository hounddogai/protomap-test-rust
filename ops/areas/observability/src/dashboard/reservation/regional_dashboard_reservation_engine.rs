use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional dashboard reservation calculation.
pub trait RegionalDashboardReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalDashboardReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalDashboardReservationAdjuster {
    pub step: f64,
}

impl RegionalDashboardReservationModifier for RoundingRegionalDashboardReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalDashboardReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalDashboardReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalDashboardReservationModifier for FlaggedRegionalDashboardReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalDashboardReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRegionalDashboardReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalDashboardReservationModifier for RegionRegionalDashboardReservationAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RegionalDashboardReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardReservationMarginOffsetAdjuster {
    pub weight: f64,
}

impl RegionalDashboardReservationModifier for RegionalDashboardReservationMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalDashboardReservation, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalDashboardReservationAdjuster {
    pub percent: f64,
}

impl RegionalDashboardReservationModifier for PercentRegionalDashboardReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalDashboardReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalDashboardReservationAdjuster {
    pub minimum: f64,
}

impl RegionalDashboardReservationModifier for MinimumRegionalDashboardReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalDashboardReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional dashboard reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalDashboardReservationEngine {
    pub steps: Vec<Rc<dyn RegionalDashboardReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalDashboardReservationEngine {
    pub fn run(&mut self, item: &RegionalDashboardReservation) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalDashboardReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_dashboard_reservation_engine(flags: &flags::Registry) -> RegionalDashboardReservationEngine {
    RegionalDashboardReservationEngine {
        steps: vec![
            Rc::new(RoundingRegionalDashboardReservationAdjuster { step: 0.5 })
                as Rc<dyn RegionalDashboardReservationModifier>,
            Rc::new(FlaggedRegionalDashboardReservationAdjuster {
                flags: flags.clone(),
                flag: "regional-dashboard-reservation-adaptive".to_string(),
                percent: 6.3,
            }),
            Rc::new(RegionRegionalDashboardReservationAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.1), ("retail".to_string(), 1.03)]),
            }),
            Rc::new(RegionalDashboardReservationMarginOffsetAdjuster { weight: 1.32 }),
            Rc::new(PercentRegionalDashboardReservationAdjuster { percent: 12.3 }),
            Rc::new(MinimumRegionalDashboardReservationAdjuster { minimum: 45.4 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 47650.6,
    }
}
