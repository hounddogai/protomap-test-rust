use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback health check template calculation.
pub trait FallbackHealthCheckTemplateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackHealthCheckTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackHealthCheckTemplateAdjuster {
    pub percent: f64,
}

impl FallbackHealthCheckTemplateStep for PercentFallbackHealthCheckTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackHealthCheckTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackHealthCheckTemplateAdjuster {
    pub step: f64,
}

impl FallbackHealthCheckTemplateStep for RoundingFallbackHealthCheckTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackHealthCheckTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackHealthCheckTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackHealthCheckTemplateStep for TieredFallbackHealthCheckTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackHealthCheckTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackHealthCheckTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackHealthCheckTemplateStep for FlaggedFallbackHealthCheckTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackHealthCheckTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckTemplateRetriesOffsetAdjuster {
    pub weight: f64,
}

impl FallbackHealthCheckTemplateStep for FallbackHealthCheckTemplateRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackHealthCheckTemplate, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackHealthCheckTemplateAdjuster {
    pub cap: f64,
}

impl FallbackHealthCheckTemplateStep for CapFallbackHealthCheckTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackHealthCheckTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback health check template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackHealthCheckTemplateEngine {
    pub steps: Vec<Rc<dyn FallbackHealthCheckTemplateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackHealthCheckTemplateEngine {
    pub fn run(&mut self, item: &FallbackHealthCheckTemplate) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackHealthCheckTemplateStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackHealthCheckTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_health_check_template_engine(flags: &flags::Registry) -> FallbackHealthCheckTemplateEngine {
    FallbackHealthCheckTemplateEngine {
        steps: vec![
            Rc::new(PercentFallbackHealthCheckTemplateAdjuster { percent: 14.4 })
                as Rc<dyn FallbackHealthCheckTemplateStep>,
            Rc::new(RoundingFallbackHealthCheckTemplateAdjuster { step: 5.0 }),
            Rc::new(TieredFallbackHealthCheckTemplateAdjuster {
                bounds: vec![832.0, 6260.0, 6727.0, 6930.0],
                rates: vec![0.83, 0.92, 0.85, 0.83],
            }),
            Rc::new(FlaggedFallbackHealthCheckTemplateAdjuster {
                flags: flags.clone(),
                flag: "fallback-health-check-template-regional".to_string(),
                percent: 1.2,
            }),
            Rc::new(FallbackHealthCheckTemplateRetriesOffsetAdjuster { weight: 0.51 }),
            Rc::new(CapFallbackHealthCheckTemplateAdjuster { cap: 8137.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 99260.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_health_check_template_engine(&flags::new_registry());
        let item = new_fallback_health_check_template("code-369", "title-983");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
