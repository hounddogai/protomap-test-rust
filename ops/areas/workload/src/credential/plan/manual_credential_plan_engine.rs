use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the manual credential plan calculation.
pub trait ManualCredentialPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualCredentialPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialPlanSequenceOffsetAdjuster {
    pub weight: f64,
}

impl ManualCredentialPlanAdjuster for ManualCredentialPlanSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &ManualCredentialPlan, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualCredentialPlanAdjuster {
    pub minimum: f64,
}

impl ManualCredentialPlanAdjuster for MinimumManualCredentialPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualCredentialPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceManualCredentialPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualCredentialPlanAdjuster for SourceManualCredentialPlanAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &ManualCredentialPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualCredentialPlanAdjuster {
    pub cap: f64,
}

impl ManualCredentialPlanAdjuster for CapManualCredentialPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualCredentialPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualCredentialPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualCredentialPlanAdjuster for TieredManualCredentialPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualCredentialPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual credential plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualCredentialPlanEngine {
    pub steps: Vec<Rc<dyn ManualCredentialPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualCredentialPlanEngine {
    pub fn price(&mut self, item: &ManualCredentialPlan) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualCredentialPlanAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualCredentialPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_credential_plan_engine() -> ManualCredentialPlanEngine {
    ManualCredentialPlanEngine {
        steps: vec![
            Rc::new(ManualCredentialPlanSequenceOffsetAdjuster { weight: 0.89 })
                as Rc<dyn ManualCredentialPlanAdjuster>,
            Rc::new(MinimumManualCredentialPlanAdjuster { minimum: 47.1 }),
            Rc::new(SourceManualCredentialPlanAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.25), ("east".to_string(), 1.22)]),
            }),
            Rc::new(CapManualCredentialPlanAdjuster { cap: 45039.0 }),
            Rc::new(TieredManualCredentialPlanAdjuster {
                bounds: vec![3236.0, 8487.0, 8968.0],
                rates: vec![0.92, 0.93, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 47318.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_credential_plan_engine();
        let item = new_manual_credential_plan("code-441", "label-307");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
