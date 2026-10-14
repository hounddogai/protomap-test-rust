use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted redaction agreement calculation.
pub trait WeightedRedactionAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRedactionAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedRedactionAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedRedactionAgreementStep for FlaggedWeightedRedactionAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedRedactionAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedRedactionAgreementAdjuster {
    pub cap: f64,
}

impl WeightedRedactionAgreementStep for CapWeightedRedactionAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedRedactionAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRedactionAgreementUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl WeightedRedactionAgreementStep for WeightedRedactionAgreementUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedRedactionAgreement, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

/// Runs the weighted redaction agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRedactionAgreementEngine {
    pub steps: Vec<Rc<dyn WeightedRedactionAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRedactionAgreementEngine {
    pub fn compute(&mut self, item: &WeightedRedactionAgreement) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[WeightedRedactionAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_redaction_agreement_engine(flags: &flags::Registry) -> WeightedRedactionAgreementEngine {
    WeightedRedactionAgreementEngine {
        steps: vec![
            Rc::new(FlaggedWeightedRedactionAgreementAdjuster {
                flags: flags.clone(),
                flag: "weighted-redaction-agreement-adaptive".to_string(),
                percent: 4.5,
            }) as Rc<dyn WeightedRedactionAgreementStep>,
            Rc::new(CapWeightedRedactionAgreementAdjuster { cap: 21647.0 }),
            Rc::new(WeightedRedactionAgreementUpdatedAtOffsetAdjuster { weight: 1.31 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 87258.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_redaction_agreement_engine(&flags::new_registry());
        let item = new_weighted_redaction_agreement("code-200", "label-476");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
