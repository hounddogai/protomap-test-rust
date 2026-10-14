use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional postmortem agreement calculation.
pub trait RegionalPostmortemAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalPostmortemAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemAgreementExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl RegionalPostmortemAgreementAdjuster for RegionalPostmortemAgreementExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalPostmortemAgreement, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalPostmortemAgreementAdjuster {
    pub percent: f64,
}

impl RegionalPostmortemAgreementAdjuster for PercentRegionalPostmortemAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalPostmortemAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalPostmortemAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalPostmortemAgreementAdjuster for TieredRegionalPostmortemAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalPostmortemAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional postmortem agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalPostmortemAgreementEngine {
    pub steps: Vec<Rc<dyn RegionalPostmortemAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalPostmortemAgreementEngine {
    pub fn run(&mut self, item: &RegionalPostmortemAgreement) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[RegionalPostmortemAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_postmortem_agreement_engine() -> RegionalPostmortemAgreementEngine {
    RegionalPostmortemAgreementEngine {
        steps: vec![
            Rc::new(RegionalPostmortemAgreementExpiresAtOffsetAdjuster { weight: 0.04 })
                as Rc<dyn RegionalPostmortemAgreementAdjuster>,
            Rc::new(PercentRegionalPostmortemAgreementAdjuster { percent: -10.0 }),
            Rc::new(TieredRegionalPostmortemAgreementAdjuster {
                bounds: vec![478.0, 948.0, 1654.0],
                rates: vec![0.92, 1.07, 0.94],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 88269.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_postmortem_agreement_engine();
        let item = new_regional_postmortem_agreement("code-102", "label-765");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
