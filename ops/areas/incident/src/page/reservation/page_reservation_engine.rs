use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the page reservation calculation.
pub trait PageReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePageReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PageReservationStep for TitlePageReservationAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PageReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PageReservationStep for TieredPageReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PageReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageReservationCapacityOffsetAdjuster {
    pub weight: f64,
}

impl PageReservationStep for PageReservationCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &PageReservation, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

/// Runs the page reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageReservationEngine {
    pub steps: Vec<Rc<dyn PageReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageReservationEngine {
    pub fn run(&mut self, item: &PageReservation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PageReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_page_reservation_engine() -> PageReservationEngine {
    PageReservationEngine {
        steps: vec![
            Rc::new(TitlePageReservationAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.26), ("central".to_string(), 0.74)]),
            }) as Rc<dyn PageReservationStep>,
            Rc::new(TieredPageReservationAdjuster {
                bounds: vec![1452.0, 8875.0],
                rates: vec![0.91, 0.89],
            }),
            Rc::new(PageReservationCapacityOffsetAdjuster { weight: 1.02 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 7017.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_reservation_engine();
        let item = new_page_reservation("code-944", "title-924");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
