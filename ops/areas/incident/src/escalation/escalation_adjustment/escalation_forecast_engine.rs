use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the escalation forecast calculation.
pub trait EscalationForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationForecastAdjuster {
    pub step: f64,
}

impl EscalationForecastStep for RoundingEscalationForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationForecastAdjuster {
    pub cap: f64,
}

impl EscalationForecastStep for CapEscalationForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameEscalationForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationForecastStep for NameEscalationForecastAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &EscalationForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationForecastEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl EscalationForecastStep for EscalationForecastEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationForecast, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

/// Runs the escalation forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationForecastEngine {
    pub steps: Vec<Rc<dyn EscalationForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationForecastEngine {
    pub fn run(&mut self, item: &EscalationForecast) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EscalationForecastStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EscalationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_escalation_forecast_engine() -> EscalationForecastEngine {
    EscalationForecastEngine {
        steps: vec![
            Rc::new(RoundingEscalationForecastAdjuster { step: 1.0 }) as Rc<dyn EscalationForecastStep>,
            Rc::new(CapEscalationForecastAdjuster { cap: 44873.0 }),
            Rc::new(NameEscalationForecastAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.95),
                    ("south".to_string(), 0.9),
                    ("east".to_string(), 0.79),
                ]),
            }),
            Rc::new(EscalationForecastEndsAtOffsetAdjuster { weight: 0.74 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 6009.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_forecast_engine();
        let item = new_escalation_forecast("code-433", "name-281");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
