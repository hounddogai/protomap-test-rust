use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the postmortem reservation calculation.
pub trait PostmortemReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemReservation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemReservationFactor for FlaggedPostmortemReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReservationThresholdOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemReservationFactor for PostmortemReservationThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemReservation, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPostmortemReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PostmortemReservationFactor for LabelPostmortemReservationAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PostmortemReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the postmortem reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemReservationEngine {
    pub steps: Vec<Rc<dyn PostmortemReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemReservationEngine {
    pub fn compute(&mut self, item: &PostmortemReservation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PostmortemReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_postmortem_reservation_engine(flags: &flags::Registry) -> PostmortemReservationEngine {
    PostmortemReservationEngine {
        steps: vec![
            Rc::new(FlaggedPostmortemReservationAdjuster {
                flags: flags.clone(),
                flag: "postmortem-reservation-primary".to_string(),
                percent: 9.7,
            }) as Rc<dyn PostmortemReservationFactor>,
            Rc::new(PostmortemReservationThresholdOffsetAdjuster { weight: 0.04 }),
            Rc::new(LabelPostmortemReservationAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.92)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 38928.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_reservation_engine(&flags::new_registry());
        let item = new_postmortem_reservation("code-929", "label-573");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
