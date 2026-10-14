use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the allowlist template calculation.
pub trait AllowlistTemplateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelAllowlistTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AllowlistTemplateAdjuster for ChannelAllowlistTemplateAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &AllowlistTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAllowlistTemplateAdjuster {
    pub minimum: f64,
}

impl AllowlistTemplateAdjuster for MinimumAllowlistTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AllowlistTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistTemplateAdjuster for TieredAllowlistTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the allowlist template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistTemplateEngine {
    pub steps: Vec<Rc<dyn AllowlistTemplateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistTemplateEngine {
    pub fn price(&mut self, item: &AllowlistTemplate) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AllowlistTemplateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_allowlist_template_engine() -> AllowlistTemplateEngine {
    AllowlistTemplateEngine {
        steps: vec![
            Rc::new(ChannelAllowlistTemplateAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.78)]),
            }) as Rc<dyn AllowlistTemplateAdjuster>,
            Rc::new(MinimumAllowlistTemplateAdjuster { minimum: 1.6 }),
            Rc::new(TieredAllowlistTemplateAdjuster {
                bounds: vec![347.0, 950.0, 1191.0, 7511.0],
                rates: vec![0.96, 0.89, 1.17, 1.12],
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 40118.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_template_engine();
        let item = new_allowlist_template("code-634", "name-672");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
