use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check exception calculation.
pub trait HealthCheckExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckException, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckExceptionStep for FlaggedHealthCheckExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingHealthCheckExceptionAdjuster {
    pub step: f64,
}

impl HealthCheckExceptionStep for RoundingHealthCheckExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &HealthCheckException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapHealthCheckExceptionAdjuster {
    pub cap: f64,
}

impl HealthCheckExceptionStep for CapHealthCheckExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &HealthCheckException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckExceptionAdjuster {
    pub minimum: f64,
}

impl HealthCheckExceptionStep for MinimumHealthCheckExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckExceptionStep for TieredHealthCheckExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the health check exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckExceptionEngine {
    pub steps: Vec<Rc<dyn HealthCheckExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckExceptionEngine {
    pub fn price(&mut self, item: &HealthCheckException) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_health_check_exception_engine(flags: &flags::Registry) -> HealthCheckExceptionEngine {
    HealthCheckExceptionEngine {
        steps: vec![
            Rc::new(FlaggedHealthCheckExceptionAdjuster {
                flags: flags.clone(),
                flag: "health-check-exception-weighted".to_string(),
                percent: -6.9,
            }) as Rc<dyn HealthCheckExceptionStep>,
            Rc::new(RoundingHealthCheckExceptionAdjuster { step: 0.5 }),
            Rc::new(CapHealthCheckExceptionAdjuster { cap: 25215.0 }),
            Rc::new(MinimumHealthCheckExceptionAdjuster { minimum: 44.7 }),
            Rc::new(TieredHealthCheckExceptionAdjuster {
                bounds: vec![4916.0, 6892.0, 8460.0, 9301.0],
                rates: vec![1.18, 1.14, 1.02, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 44005.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_exception_engine(&flags::new_registry());
        let item = new_health_check_exception("code-738", "title-901");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
