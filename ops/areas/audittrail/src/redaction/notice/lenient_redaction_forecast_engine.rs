use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient redaction forecast calculation.
pub trait LenientRedactionForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientRedactionForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitLenientRedactionForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientRedactionForecastModifier for UnitLenientRedactionForecastAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &LenientRedactionForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientRedactionForecastAdjuster {
    pub percent: f64,
}

impl LenientRedactionForecastModifier for PercentLenientRedactionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientRedactionForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientRedactionForecastAdjuster {
    pub minimum: f64,
}

impl LenientRedactionForecastModifier for MinimumLenientRedactionForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientRedactionForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientRedactionForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientRedactionForecastModifier for TieredLenientRedactionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientRedactionForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient redaction forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientRedactionForecastEngine {
    pub steps: Vec<Rc<dyn LenientRedactionForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientRedactionForecastEngine {
    pub fn run(&mut self, item: &LenientRedactionForecast) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[LenientRedactionForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_redaction_forecast_engine() -> LenientRedactionForecastEngine {
    LenientRedactionForecastEngine {
        steps: vec![
            Rc::new(UnitLenientRedactionForecastAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.95),
                    ("west".to_string(), 0.76),
                    ("south".to_string(), 0.79),
                ]),
            }) as Rc<dyn LenientRedactionForecastModifier>,
            Rc::new(PercentLenientRedactionForecastAdjuster { percent: 2.0 }),
            Rc::new(MinimumLenientRedactionForecastAdjuster { minimum: 49.9 }),
            Rc::new(TieredLenientRedactionForecastAdjuster {
                bounds: vec![645.0, 1486.0, 1994.0, 9204.0],
                rates: vec![1.08, 1.15, 0.97, 0.87],
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 39017.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_redaction_forecast_engine();
        let item = new_lenient_redaction_forecast("code-12", "label-678");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
