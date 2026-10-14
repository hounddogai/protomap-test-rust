use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered peering agreement calculation.
pub trait TieredPeeringAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredPeeringAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleTieredPeeringAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredPeeringAgreementAdjuster for TitleTieredPeeringAgreementAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &TieredPeeringAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredPeeringAgreementAdjuster {
    pub percent: f64,
}

impl TieredPeeringAgreementAdjuster for PercentTieredPeeringAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredPeeringAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAgreementThresholdOffsetAdjuster {
    pub weight: f64,
}

impl TieredPeeringAgreementAdjuster for TieredPeeringAgreementThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &TieredPeeringAgreement, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredPeeringAgreementAdjuster {
    pub cap: f64,
}

impl TieredPeeringAgreementAdjuster for CapTieredPeeringAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredPeeringAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the tiered peering agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredPeeringAgreementEngine {
    pub steps: Vec<Rc<dyn TieredPeeringAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredPeeringAgreementEngine {
    pub fn price(&mut self, item: &TieredPeeringAgreement) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredPeeringAgreementAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredPeeringAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_peering_agreement_engine() -> TieredPeeringAgreementEngine {
    TieredPeeringAgreementEngine {
        steps: vec![
            Rc::new(TitleTieredPeeringAgreementAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.87), ("south".to_string(), 0.7)]),
            }) as Rc<dyn TieredPeeringAgreementAdjuster>,
            Rc::new(PercentTieredPeeringAgreementAdjuster { percent: -1.1 }),
            Rc::new(TieredPeeringAgreementThresholdOffsetAdjuster { weight: 1.3 }),
            Rc::new(CapTieredPeeringAgreementAdjuster { cap: 47154.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 49733.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_peering_agreement_engine();
        let item = new_tiered_peering_agreement("code-496", "title-873");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
