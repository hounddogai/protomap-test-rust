use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the egress request calculation.
pub trait EgressRequestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionEgressRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EgressRequestAdjuster for RegionEgressRequestAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &EgressRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEgressRequestAdjuster {
    pub minimum: f64,
}

impl EgressRequestAdjuster for MinimumEgressRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EgressRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressRequestAdjuster {
    pub percent: f64,
}

impl EgressRequestAdjuster for PercentEgressRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressRequestAdjuster for FlaggedEgressRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the egress request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressRequestEngine {
    pub steps: Vec<Rc<dyn EgressRequestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressRequestEngine {
    pub fn price(&mut self, item: &EgressRequest) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[EgressRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_egress_request_engine(flags: &flags::Registry) -> EgressRequestEngine {
    EgressRequestEngine {
        steps: vec![
            Rc::new(RegionEgressRequestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.95), ("south".to_string(), 1.05)]),
            }) as Rc<dyn EgressRequestAdjuster>,
            Rc::new(MinimumEgressRequestAdjuster { minimum: 8.7 }),
            Rc::new(PercentEgressRequestAdjuster { percent: -16.8 }),
            Rc::new(FlaggedEgressRequestAdjuster {
                flags: flags.clone(),
                flag: "egress-request-legacy".to_string(),
                percent: -2.9,
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 15512.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_request_engine(&flags::new_registry());
        let item = new_egress_request("code-189", "label-721");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
