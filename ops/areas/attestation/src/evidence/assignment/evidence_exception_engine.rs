use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the evidence exception calculation.
pub trait EvidenceExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceException, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceExceptionAdjuster for FlaggedEvidenceExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceExceptionAdjuster for TieredEvidenceExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryEvidenceExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceExceptionAdjuster for CategoryEvidenceExceptionAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &EvidenceException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the evidence exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceExceptionEngine {
    pub steps: Vec<Rc<dyn EvidenceExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceExceptionEngine {
    pub fn price(&mut self, item: &EvidenceException) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_evidence_exception_engine(flags: &flags::Registry) -> EvidenceExceptionEngine {
    EvidenceExceptionEngine {
        steps: vec![
            Rc::new(FlaggedEvidenceExceptionAdjuster {
                flags: flags.clone(),
                flag: "evidence-exception-global".to_string(),
                percent: 0.8,
            }) as Rc<dyn EvidenceExceptionAdjuster>,
            Rc::new(TieredEvidenceExceptionAdjuster {
                bounds: vec![467.0, 9273.0],
                rates: vec![0.91, 1.0],
            }),
            Rc::new(CategoryEvidenceExceptionAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.29), ("central".to_string(), 1.25)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 66106.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_exception_engine(&flags::new_registry());
        let item = new_evidence_exception("code-20", "label-480");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
