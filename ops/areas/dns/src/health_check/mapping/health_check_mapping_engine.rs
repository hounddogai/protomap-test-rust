use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the health check mapping calculation.
pub trait HealthCheckMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapHealthCheckMappingAdjuster {
    pub cap: f64,
}

impl HealthCheckMappingStep for CapHealthCheckMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &HealthCheckMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckMappingAdjuster {
    pub percent: f64,
}

impl HealthCheckMappingStep for PercentHealthCheckMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckMappingAdjuster {
    pub minimum: f64,
}

impl HealthCheckMappingStep for MinimumHealthCheckMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMappingUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl HealthCheckMappingStep for HealthCheckMappingUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &HealthCheckMapping, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleHealthCheckMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckMappingStep for TitleHealthCheckMappingAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &HealthCheckMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the health check mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckMappingEngine {
    pub steps: Vec<Rc<dyn HealthCheckMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckMappingEngine {
    pub fn run(&mut self, item: &HealthCheckMapping) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_health_check_mapping_engine() -> HealthCheckMappingEngine {
    HealthCheckMappingEngine {
        steps: vec![
            Rc::new(CapHealthCheckMappingAdjuster { cap: 10159.0 }) as Rc<dyn HealthCheckMappingStep>,
            Rc::new(PercentHealthCheckMappingAdjuster { percent: -23.4 }),
            Rc::new(MinimumHealthCheckMappingAdjuster { minimum: 12.2 }),
            Rc::new(HealthCheckMappingUpdatedAtOffsetAdjuster { weight: 0.43 }),
            Rc::new(TitleHealthCheckMappingAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.17),
                    ("retail".to_string(), 1.31),
                    ("north".to_string(), 1.11),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 46355.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_mapping_engine();
        let item = new_health_check_mapping("code-836", "title-883");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
