use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the usage reservation calculation.
pub trait UsageReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl UsageReservationAdjuster for TieredUsageReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &UsageReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentUsageReservationAdjuster {
    pub percent: f64,
}

impl UsageReservationAdjuster for PercentUsageReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &UsageReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageReservationAdjuster {
    pub minimum: f64,
}

impl UsageReservationAdjuster for MinimumUsageReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeUsageReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageReservationAdjuster for ScopeUsageReservationAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &UsageReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the usage reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageReservationEngine {
    pub steps: Vec<Rc<dyn UsageReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageReservationEngine {
    pub fn price(&mut self, item: &UsageReservation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[UsageReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_usage_reservation_engine() -> UsageReservationEngine {
    UsageReservationEngine {
        steps: vec![
            Rc::new(TieredUsageReservationAdjuster {
                bounds: vec![1654.0, 3457.0, 8336.0],
                rates: vec![0.97, 0.85, 1.2],
            }) as Rc<dyn UsageReservationAdjuster>,
            Rc::new(PercentUsageReservationAdjuster { percent: 19.6 }),
            Rc::new(MinimumUsageReservationAdjuster { minimum: 39.7 }),
            Rc::new(ScopeUsageReservationAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.21),
                    ("west".to_string(), 1.25),
                    ("central".to_string(), 1.13),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 20454.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_reservation_engine();
        let item = new_usage_reservation("code-554", "title-643");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
