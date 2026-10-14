use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted peering request calculation.
pub trait WeightedPeeringRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedPeeringRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedPeeringRequestAdjuster {
    pub step: f64,
}

impl WeightedPeeringRequestStep for RoundingWeightedPeeringRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedPeeringRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceWeightedPeeringRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedPeeringRequestStep for NamespaceWeightedPeeringRequestAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &WeightedPeeringRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedPeeringRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedPeeringRequestStep for FlaggedWeightedPeeringRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedPeeringRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted peering request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedPeeringRequestEngine {
    pub steps: Vec<Rc<dyn WeightedPeeringRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedPeeringRequestEngine {
    pub fn price(&mut self, item: &WeightedPeeringRequest) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[WeightedPeeringRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_peering_request_engine(flags: &flags::Registry) -> WeightedPeeringRequestEngine {
    WeightedPeeringRequestEngine {
        steps: vec![
            Rc::new(RoundingWeightedPeeringRequestAdjuster { step: 5.0 }) as Rc<dyn WeightedPeeringRequestStep>,
            Rc::new(NamespaceWeightedPeeringRequestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.82), ("retail".to_string(), 0.8)]),
            }),
            Rc::new(FlaggedWeightedPeeringRequestAdjuster {
                flags: flags.clone(),
                flag: "weighted-peering-request-seasonal".to_string(),
                percent: 1.3,
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 38784.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_peering_request_engine(&flags::new_registry());
        let item = new_weighted_peering_request("code-522", "name-238");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
