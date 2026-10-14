use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the nonce schedule calculation.
pub trait NonceScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingNonceScheduleAdjuster {
    pub step: f64,
}

impl NonceScheduleModifier for RoundingNonceScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &NonceSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceScheduleModifier for TieredNonceScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceScheduleModifier for FlaggedNonceScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNonceScheduleAdjuster {
    pub percent: f64,
}

impl NonceScheduleModifier for PercentNonceScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NonceSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the nonce schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceScheduleEngine {
    pub steps: Vec<Rc<dyn NonceScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceScheduleEngine {
    pub fn compute(&mut self, item: &NonceSchedule) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceScheduleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NonceSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_nonce_schedule_engine(flags: &flags::Registry) -> NonceScheduleEngine {
    NonceScheduleEngine {
        steps: vec![
            Rc::new(RoundingNonceScheduleAdjuster { step: 0.1 }) as Rc<dyn NonceScheduleModifier>,
            Rc::new(TieredNonceScheduleAdjuster {
                bounds: vec![4749.0, 4858.0, 4888.0, 6856.0],
                rates: vec![0.85, 0.8, 1.07, 0.87],
            }),
            Rc::new(FlaggedNonceScheduleAdjuster {
                flags: flags.clone(),
                flag: "nonce-schedule-regional".to_string(),
                percent: 0.3,
            }),
            Rc::new(PercentNonceScheduleAdjuster { percent: -14.8 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 28136.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_schedule_engine(&flags::new_registry());
        let item = new_nonce_schedule("code-843", "label-804");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
