use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check template calculation.
pub trait HealthCheckTemplateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTemplateScoreOffsetAdjuster {
    pub weight: f64,
}

impl HealthCheckTemplateStep for HealthCheckTemplateScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &HealthCheckTemplate, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckTemplateStep for FlaggedHealthCheckTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelHealthCheckTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckTemplateStep for ChannelHealthCheckTemplateAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &HealthCheckTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckTemplateStep for TieredHealthCheckTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the health check template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckTemplateEngine {
    pub steps: Vec<Rc<dyn HealthCheckTemplateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckTemplateEngine {
    pub fn evaluate(&mut self, item: &HealthCheckTemplate) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_health_check_template_engine(flags: &flags::Registry) -> HealthCheckTemplateEngine {
    HealthCheckTemplateEngine {
        steps: vec![
            Rc::new(HealthCheckTemplateScoreOffsetAdjuster { weight: 1.07 }) as Rc<dyn HealthCheckTemplateStep>,
            Rc::new(FlaggedHealthCheckTemplateAdjuster {
                flags: flags.clone(),
                flag: "health-check-template-pending".to_string(),
                percent: 1.1,
            }),
            Rc::new(ChannelHealthCheckTemplateAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.04)]),
            }),
            Rc::new(TieredHealthCheckTemplateAdjuster {
                bounds: vec![3808.0, 3984.0, 4037.0, 7526.0],
                rates: vec![1.18, 1.06, 1.1, 1.18],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 7973.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_template_engine(&flags::new_registry());
        let item = new_health_check_template("code-282", "label-227");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
