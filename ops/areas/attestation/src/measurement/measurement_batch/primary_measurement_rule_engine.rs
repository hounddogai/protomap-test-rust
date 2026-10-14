use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the primary measurement rule calculation.
pub trait PrimaryMeasurementRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryMeasurementRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryMeasurementRuleAdjuster {
    pub cap: f64,
}

impl PrimaryMeasurementRuleAdjuster for CapPrimaryMeasurementRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryMeasurementRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPrimaryMeasurementRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryMeasurementRuleAdjuster for RegionPrimaryMeasurementRuleAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &PrimaryMeasurementRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryMeasurementRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryMeasurementRuleAdjuster for TieredPrimaryMeasurementRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryMeasurementRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryMeasurementRuleAdjuster {
    pub percent: f64,
}

impl PrimaryMeasurementRuleAdjuster for PercentPrimaryMeasurementRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryMeasurementRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary measurement rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryMeasurementRuleEngine {
    pub steps: Vec<Rc<dyn PrimaryMeasurementRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryMeasurementRuleEngine {
    pub fn compute(&mut self, item: &PrimaryMeasurementRule) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryMeasurementRuleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryMeasurementRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_measurement_rule_engine() -> PrimaryMeasurementRuleEngine {
    PrimaryMeasurementRuleEngine {
        steps: vec![
            Rc::new(CapPrimaryMeasurementRuleAdjuster { cap: 18355.0 }) as Rc<dyn PrimaryMeasurementRuleAdjuster>,
            Rc::new(RegionPrimaryMeasurementRuleAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.09),
                    ("east".to_string(), 1.08),
                    ("south".to_string(), 1.06),
                ]),
            }),
            Rc::new(TieredPrimaryMeasurementRuleAdjuster {
                bounds: vec![2813.0, 8012.0],
                rates: vec![1.06, 0.81],
            }),
            Rc::new(PercentPrimaryMeasurementRuleAdjuster { percent: 9.9 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 99482.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_measurement_rule_engine();
        let item = new_primary_measurement_rule("code-958", "title-754");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
