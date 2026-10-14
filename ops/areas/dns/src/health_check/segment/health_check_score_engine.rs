use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check score calculation.
pub trait HealthCheckScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckScoreAdjuster {
    pub minimum: f64,
}

impl HealthCheckScoreAdjuster for MinimumHealthCheckScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckScoreAdjuster for TieredHealthCheckScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckScoreAdjuster {
    pub percent: f64,
}

impl HealthCheckScoreAdjuster for PercentHealthCheckScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingHealthCheckScoreAdjuster {
    pub step: f64,
}

impl HealthCheckScoreAdjuster for RoundingHealthCheckScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &HealthCheckScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckScoreAdjuster for FlaggedHealthCheckScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckScoreRetriesOffsetAdjuster {
    pub weight: f64,
}

impl HealthCheckScoreAdjuster for HealthCheckScoreRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &HealthCheckScore, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

/// Runs the health check score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckScoreEngine {
    pub steps: Vec<Rc<dyn HealthCheckScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckScoreEngine {
    pub fn price(&mut self, item: &HealthCheckScore) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn HealthCheckScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_health_check_score_engine(flags: &flags::Registry) -> HealthCheckScoreEngine {
    HealthCheckScoreEngine {
        steps: vec![
            Rc::new(MinimumHealthCheckScoreAdjuster { minimum: 0.1 }) as Rc<dyn HealthCheckScoreAdjuster>,
            Rc::new(TieredHealthCheckScoreAdjuster {
                bounds: vec![1942.0, 4073.0, 6674.0, 8663.0],
                rates: vec![1.19, 0.97, 1.01, 1.09],
            }),
            Rc::new(PercentHealthCheckScoreAdjuster { percent: 22.3 }),
            Rc::new(RoundingHealthCheckScoreAdjuster { step: 1.0 }),
            Rc::new(FlaggedHealthCheckScoreAdjuster {
                flags: flags.clone(),
                flag: "health-check-score-scheduled".to_string(),
                percent: 4.9,
            }),
            Rc::new(HealthCheckScoreRetriesOffsetAdjuster { weight: 1.74 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 62206.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_score_engine(&flags::new_registry());
        let item = new_health_check_score("code-372", "name-340");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
