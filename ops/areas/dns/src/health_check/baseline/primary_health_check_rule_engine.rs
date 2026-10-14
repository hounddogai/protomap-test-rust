use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary health check rule calculation.
pub trait PrimaryHealthCheckRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryHealthCheckRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryHealthCheckRuleAdjuster {
    pub percent: f64,
}

impl PrimaryHealthCheckRuleStep for PercentPrimaryHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryHealthCheckRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryHealthCheckRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryHealthCheckRuleStep for TieredPrimaryHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryHealthCheckRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryHealthCheckRuleAdjuster {
    pub cap: f64,
}

impl PrimaryHealthCheckRuleStep for CapPrimaryHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryHealthCheckRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryHealthCheckRuleAdjuster {
    pub minimum: f64,
}

impl PrimaryHealthCheckRuleStep for MinimumPrimaryHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryHealthCheckRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckRuleBurstOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryHealthCheckRuleStep for PrimaryHealthCheckRuleBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryHealthCheckRule, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryHealthCheckRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryHealthCheckRuleStep for FlaggedPrimaryHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryHealthCheckRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary health check rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryHealthCheckRuleEngine {
    pub steps: Vec<Rc<dyn PrimaryHealthCheckRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryHealthCheckRuleEngine {
    pub fn price(&mut self, item: &PrimaryHealthCheckRule) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryHealthCheckRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_health_check_rule_engine(flags: &flags::Registry) -> PrimaryHealthCheckRuleEngine {
    PrimaryHealthCheckRuleEngine {
        steps: vec![
            Rc::new(PercentPrimaryHealthCheckRuleAdjuster { percent: 24.8 }) as Rc<dyn PrimaryHealthCheckRuleStep>,
            Rc::new(TieredPrimaryHealthCheckRuleAdjuster {
                bounds: vec![981.0, 7584.0],
                rates: vec![0.88, 1.01],
            }),
            Rc::new(CapPrimaryHealthCheckRuleAdjuster { cap: 38510.0 }),
            Rc::new(MinimumPrimaryHealthCheckRuleAdjuster { minimum: 43.5 }),
            Rc::new(PrimaryHealthCheckRuleBurstOffsetAdjuster { weight: 1.38 }),
            Rc::new(FlaggedPrimaryHealthCheckRuleAdjuster {
                flags: flags.clone(),
                flag: "primary-health-check-rule-rolling".to_string(),
                percent: -9.9,
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 79879.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_health_check_rule_engine(&flags::new_registry());
        let item = new_primary_health_check_rule("code-710", "name-185");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
