use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the nonce template calculation.
pub trait NonceTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceTemplateModifier for TieredNonceTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceTemplateModifier for FlaggedNonceTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNonceTemplateAdjuster {
    pub minimum: f64,
}

impl NonceTemplateModifier for MinimumNonceTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NonceTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTemplateMaximumOffsetAdjuster {
    pub weight: f64,
}

impl NonceTemplateModifier for NonceTemplateMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &NonceTemplate, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNonceTemplateAdjuster {
    pub percent: f64,
}

impl NonceTemplateModifier for PercentNonceTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NonceTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the nonce template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceTemplateEngine {
    pub steps: Vec<Rc<dyn NonceTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceTemplateEngine {
    pub fn compute(&mut self, item: &NonceTemplate) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[NonceTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_nonce_template_engine(flags: &flags::Registry) -> NonceTemplateEngine {
    NonceTemplateEngine {
        steps: vec![
            Rc::new(TieredNonceTemplateAdjuster {
                bounds: vec![5641.0, 6032.0, 9702.0, 9880.0],
                rates: vec![1.15, 0.82, 0.91, 0.87],
            }) as Rc<dyn NonceTemplateModifier>,
            Rc::new(FlaggedNonceTemplateAdjuster {
                flags: flags.clone(),
                flag: "nonce-template-fallback".to_string(),
                percent: -1.1,
            }),
            Rc::new(MinimumNonceTemplateAdjuster { minimum: 43.8 }),
            Rc::new(NonceTemplateMaximumOffsetAdjuster { weight: 0.25 }),
            Rc::new(PercentNonceTemplateAdjuster { percent: 20.6 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 80692.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_template_engine(&flags::new_registry());
        let item = new_nonce_template("code-814", "name-276");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
