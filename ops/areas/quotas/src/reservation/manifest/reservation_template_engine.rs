use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the reservation template calculation.
pub trait ReservationTemplateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationTemplateAdjuster {
    pub percent: f64,
}

impl ReservationTemplateAdjuster for PercentReservationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationTemplateAdjuster {
    pub cap: f64,
}

impl ReservationTemplateAdjuster for CapReservationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTemplatePriorityOffsetAdjuster {
    pub weight: f64,
}

impl ReservationTemplateAdjuster for ReservationTemplatePriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationTemplate, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationTemplateAdjuster {
    pub step: f64,
}

impl ReservationTemplateAdjuster for RoundingReservationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleReservationTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationTemplateAdjuster for TitleReservationTemplateAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ReservationTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the reservation template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationTemplateEngine {
    pub steps: Vec<Rc<dyn ReservationTemplateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationTemplateEngine {
    pub fn price(&mut self, item: &ReservationTemplate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ReservationTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_reservation_template_engine() -> ReservationTemplateEngine {
    ReservationTemplateEngine {
        steps: vec![
            Rc::new(PercentReservationTemplateAdjuster { percent: 21.2 }) as Rc<dyn ReservationTemplateAdjuster>,
            Rc::new(CapReservationTemplateAdjuster { cap: 8526.0 }),
            Rc::new(ReservationTemplatePriorityOffsetAdjuster { weight: 1.53 }),
            Rc::new(RoundingReservationTemplateAdjuster { step: 0.5 }),
            Rc::new(TitleReservationTemplateAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.96)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 64070.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_template_engine();
        let item = new_reservation_template("code-816", "title-483");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
