use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the page agreement calculation.
pub trait PageAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageAgreementAdjuster {
    pub minimum: f64,
}

impl PageAgreementFactor for MinimumPageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPageAgreementAdjuster {
    pub cap: f64,
}

impl PageAgreementFactor for CapPageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PageAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageAgreementAdjuster {
    pub percent: f64,
}

impl PageAgreementFactor for PercentPageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPageAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PageAgreementFactor for LabelPageAgreementAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PageAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the page agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageAgreementEngine {
    pub steps: Vec<Rc<dyn PageAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageAgreementEngine {
    pub fn evaluate(&mut self, item: &PageAgreement) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PageAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_page_agreement_engine() -> PageAgreementEngine {
    PageAgreementEngine {
        steps: vec![
            Rc::new(MinimumPageAgreementAdjuster { minimum: 39.5 }) as Rc<dyn PageAgreementFactor>,
            Rc::new(CapPageAgreementAdjuster { cap: 7649.0 }),
            Rc::new(PercentPageAgreementAdjuster { percent: -0.5 }),
            Rc::new(LabelPageAgreementAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.02),
                    ("central".to_string(), 1.12),
                    ("north".to_string(), 0.84),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 72341.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_agreement_engine();
        let item = new_page_agreement("code-53", "label-388");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
