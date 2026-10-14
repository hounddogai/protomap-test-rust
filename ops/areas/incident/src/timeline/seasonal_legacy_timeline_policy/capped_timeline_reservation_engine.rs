use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped timeline reservation calculation.
pub trait CappedTimelineReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedTimelineReservation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedTimelineReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedTimelineReservationFactor for FlaggedCappedTimelineReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedTimelineReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTimelineReservationBurstOffsetAdjuster {
    pub weight: f64,
}

impl CappedTimelineReservationFactor for CappedTimelineReservationBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &CappedTimelineReservation, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedTimelineReservationAdjuster {
    pub percent: f64,
}

impl CappedTimelineReservationFactor for PercentCappedTimelineReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedTimelineReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedTimelineReservationAdjuster {
    pub cap: f64,
}

impl CappedTimelineReservationFactor for CapCappedTimelineReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedTimelineReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedTimelineReservationAdjuster {
    pub step: f64,
}

impl CappedTimelineReservationFactor for RoundingCappedTimelineReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedTimelineReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleCappedTimelineReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedTimelineReservationFactor for TitleCappedTimelineReservationAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &CappedTimelineReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the capped timeline reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedTimelineReservationEngine {
    pub steps: Vec<Rc<dyn CappedTimelineReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedTimelineReservationEngine {
    pub fn evaluate(&mut self, item: &CappedTimelineReservation) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[CappedTimelineReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_timeline_reservation_engine(flags: &flags::Registry) -> CappedTimelineReservationEngine {
    CappedTimelineReservationEngine {
        steps: vec![
            Rc::new(FlaggedCappedTimelineReservationAdjuster {
                flags: flags.clone(),
                flag: "capped-timeline-reservation-capped".to_string(),
                percent: -1.4,
            }) as Rc<dyn CappedTimelineReservationFactor>,
            Rc::new(CappedTimelineReservationBurstOffsetAdjuster { weight: 1.86 }),
            Rc::new(PercentCappedTimelineReservationAdjuster { percent: -14.1 }),
            Rc::new(CapCappedTimelineReservationAdjuster { cap: 1861.0 }),
            Rc::new(RoundingCappedTimelineReservationAdjuster { step: 0.25 }),
            Rc::new(TitleCappedTimelineReservationAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.09)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 69655.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_timeline_reservation_engine(&flags::new_registry());
        let item = new_capped_timeline_reservation("code-632", "title-840");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
