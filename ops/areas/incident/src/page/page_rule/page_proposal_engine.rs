use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the page proposal calculation.
pub trait PageProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageProposalAdjuster {
    pub percent: f64,
}

impl PageProposalModifier for PercentPageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageProposalTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl PageProposalModifier for PageProposalTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &PageProposal, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageProposalAdjuster {
    pub minimum: f64,
}

impl PageProposalModifier for MinimumPageProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPageProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PageProposalModifier for CategoryPageProposalAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PageProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PageProposalModifier for TieredPageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PageProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the page proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageProposalEngine {
    pub steps: Vec<Rc<dyn PageProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageProposalEngine {
    pub fn evaluate(&mut self, item: &PageProposal) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PageProposalModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PageProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_page_proposal_engine() -> PageProposalEngine {
    PageProposalEngine {
        steps: vec![
            Rc::new(PercentPageProposalAdjuster { percent: -24.3 }) as Rc<dyn PageProposalModifier>,
            Rc::new(PageProposalTtlSecondsOffsetAdjuster { weight: 0.4 }),
            Rc::new(MinimumPageProposalAdjuster { minimum: 42.9 }),
            Rc::new(CategoryPageProposalAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.0),
                    ("south".to_string(), 1.06),
                    ("online".to_string(), 1.31),
                ]),
            }),
            Rc::new(TieredPageProposalAdjuster {
                bounds: vec![3776.0, 7272.0],
                rates: vec![1.11, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 30582.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_proposal_engine();
        let item = new_page_proposal("code-609", "name-837");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
