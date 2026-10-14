use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check request calculation.
pub trait HealthCheckRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingHealthCheckRequestAdjuster {
    pub step: f64,
}

impl HealthCheckRequestStep for RoundingHealthCheckRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &HealthCheckRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRequestStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl HealthCheckRequestStep for HealthCheckRequestStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &HealthCheckRequest, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckRequestStep for FlaggedHealthCheckRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckRequestAdjuster {
    pub minimum: f64,
}

impl HealthCheckRequestStep for MinimumHealthCheckRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleHealthCheckRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckRequestStep for TitleHealthCheckRequestAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &HealthCheckRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapHealthCheckRequestAdjuster {
    pub cap: f64,
}

impl HealthCheckRequestStep for CapHealthCheckRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &HealthCheckRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the health check request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckRequestEngine {
    pub steps: Vec<Rc<dyn HealthCheckRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckRequestEngine {
    pub fn price(&mut self, item: &HealthCheckRequest) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_health_check_request_engine(flags: &flags::Registry) -> HealthCheckRequestEngine {
    HealthCheckRequestEngine {
        steps: vec![
            Rc::new(RoundingHealthCheckRequestAdjuster { step: 0.5 }) as Rc<dyn HealthCheckRequestStep>,
            Rc::new(HealthCheckRequestStartsAtOffsetAdjuster { weight: 1.44 }),
            Rc::new(FlaggedHealthCheckRequestAdjuster {
                flags: flags.clone(),
                flag: "health-check-request-primary".to_string(),
                percent: 4.1,
            }),
            Rc::new(MinimumHealthCheckRequestAdjuster { minimum: 46.4 }),
            Rc::new(TitleHealthCheckRequestAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.34), ("north".to_string(), 1.14)]),
            }),
            Rc::new(CapHealthCheckRequestAdjuster { cap: 21356.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 56293.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_request_engine(&flags::new_registry());
        let item = new_health_check_request("code-704", "title-529");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
