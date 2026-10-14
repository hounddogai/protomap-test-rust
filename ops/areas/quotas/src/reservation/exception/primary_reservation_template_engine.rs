use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the primary reservation template calculation.
pub trait PrimaryReservationTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryReservationTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryReservationTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryReservationTemplateFactor for TieredPrimaryReservationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryReservationTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryReservationTemplateAdjuster {
    pub cap: f64,
}

impl PrimaryReservationTemplateFactor for CapPrimaryReservationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryReservationTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryReservationTemplateAdjuster {
    pub minimum: f64,
}

impl PrimaryReservationTemplateFactor for MinimumPrimaryReservationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryReservationTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationTemplateWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryReservationTemplateFactor for PrimaryReservationTemplateWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryReservationTemplate, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the primary reservation template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryReservationTemplateEngine {
    pub steps: Vec<Rc<dyn PrimaryReservationTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryReservationTemplateEngine {
    pub fn price(&mut self, item: &PrimaryReservationTemplate) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryReservationTemplateFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryReservationTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_reservation_template_engine() -> PrimaryReservationTemplateEngine {
    PrimaryReservationTemplateEngine {
        steps: vec![
            Rc::new(TieredPrimaryReservationTemplateAdjuster {
                bounds: vec![5586.0, 7307.0, 7704.0],
                rates: vec![0.95, 0.98, 0.83],
            }) as Rc<dyn PrimaryReservationTemplateFactor>,
            Rc::new(CapPrimaryReservationTemplateAdjuster { cap: 45888.0 }),
            Rc::new(MinimumPrimaryReservationTemplateAdjuster { minimum: 10.3 }),
            Rc::new(PrimaryReservationTemplateWindowDaysOffsetAdjuster { weight: 1.29 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 48942.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_reservation_template_engine();
        let item = new_primary_reservation_template("code-201", "title-779");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
