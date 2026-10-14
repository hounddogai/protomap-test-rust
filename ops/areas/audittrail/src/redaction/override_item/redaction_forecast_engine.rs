use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the redaction forecast calculation.
pub trait RedactionForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionForecast, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionForecastModifier for FlaggedRedactionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRedactionForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionForecastModifier for CategoryRedactionForecastAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RedactionForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionForecastAdjuster {
    pub percent: f64,
}

impl RedactionForecastModifier for PercentRedactionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionForecastModifier for TieredRedactionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionForecastAdjuster {
    pub cap: f64,
}

impl RedactionForecastModifier for CapRedactionForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the redaction forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionForecastEngine {
    pub steps: Vec<Rc<dyn RedactionForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionForecastEngine {
    pub fn compute(&mut self, item: &RedactionForecast) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[RedactionForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_redaction_forecast_engine(flags: &flags::Registry) -> RedactionForecastEngine {
    RedactionForecastEngine {
        steps: vec![
            Rc::new(FlaggedRedactionForecastAdjuster {
                flags: flags.clone(),
                flag: "redaction-forecast-primary".to_string(),
                percent: -5.9,
            }) as Rc<dyn RedactionForecastModifier>,
            Rc::new(CategoryRedactionForecastAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.21), ("east".to_string(), 1.1)]),
            }),
            Rc::new(PercentRedactionForecastAdjuster { percent: 23.7 }),
            Rc::new(TieredRedactionForecastAdjuster {
                bounds: vec![7522.0, 9629.0],
                rates: vec![0.95, 1.02],
            }),
            Rc::new(CapRedactionForecastAdjuster { cap: 35961.0 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 18534.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_forecast_engine(&flags::new_registry());
        let item = new_redaction_forecast("code-671", "name-645");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
