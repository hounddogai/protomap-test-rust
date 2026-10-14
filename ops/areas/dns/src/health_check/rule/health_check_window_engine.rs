use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check window calculation.
pub trait HealthCheckWindowAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckWindowCeilingOffsetAdjuster {
    pub weight: f64,
}

impl HealthCheckWindowAdjuster for HealthCheckWindowCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &HealthCheckWindow, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckWindowAdjuster for FlaggedHealthCheckWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckWindowAdjuster {
    pub minimum: f64,
}

impl HealthCheckWindowAdjuster for MinimumHealthCheckWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckWindowAdjuster for TieredHealthCheckWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingHealthCheckWindowAdjuster {
    pub step: f64,
}

impl HealthCheckWindowAdjuster for RoundingHealthCheckWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &HealthCheckWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the health check window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckWindowEngine {
    pub steps: Vec<Rc<dyn HealthCheckWindowAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckWindowEngine {
    pub fn evaluate(&mut self, item: &HealthCheckWindow) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_health_check_window_engine(flags: &flags::Registry) -> HealthCheckWindowEngine {
    HealthCheckWindowEngine {
        steps: vec![
            Rc::new(HealthCheckWindowCeilingOffsetAdjuster { weight: 1.65 }) as Rc<dyn HealthCheckWindowAdjuster>,
            Rc::new(FlaggedHealthCheckWindowAdjuster {
                flags: flags.clone(),
                flag: "health-check-window-seasonal".to_string(),
                percent: 4.8,
            }),
            Rc::new(MinimumHealthCheckWindowAdjuster { minimum: 39.5 }),
            Rc::new(TieredHealthCheckWindowAdjuster {
                bounds: vec![84.0, 2413.0, 8944.0],
                rates: vec![1.1, 0.85, 1.03],
            }),
            Rc::new(RoundingHealthCheckWindowAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 48488.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_window_engine(&flags::new_registry());
        let item = new_health_check_window("code-975", "label-613");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
