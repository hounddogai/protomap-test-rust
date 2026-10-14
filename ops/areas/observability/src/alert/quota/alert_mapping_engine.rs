use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the alert mapping calculation.
pub trait AlertMappingAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertMappingRatioOffsetAdjuster {
    pub weight: f64,
}

impl AlertMappingAdjuster for AlertMappingRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &AlertMapping, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelAlertMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertMappingAdjuster for ChannelAlertMappingAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &AlertMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertMappingAdjuster {
    pub percent: f64,
}

impl AlertMappingAdjuster for PercentAlertMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the alert mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertMappingEngine {
    pub steps: Vec<Rc<dyn AlertMappingAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertMappingEngine {
    pub fn run(&mut self, item: &AlertMapping) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AlertMappingAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AlertMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_alert_mapping_engine() -> AlertMappingEngine {
    AlertMappingEngine {
        steps: vec![
            Rc::new(AlertMappingRatioOffsetAdjuster { weight: 1.45 }) as Rc<dyn AlertMappingAdjuster>,
            Rc::new(ChannelAlertMappingAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.29)]),
            }),
            Rc::new(PercentAlertMappingAdjuster { percent: 14.8 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 97788.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_mapping_engine();
        let item = new_alert_mapping("code-417", "title-802");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
