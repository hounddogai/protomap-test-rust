use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted dns record template calculation.
pub trait WeightedDnsRecordTemplateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedDnsRecordTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedDnsRecordTemplateAdjuster {
    pub minimum: f64,
}

impl WeightedDnsRecordTemplateAdjuster for MinimumWeightedDnsRecordTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedDnsRecordTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleWeightedDnsRecordTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedDnsRecordTemplateAdjuster for LocaleWeightedDnsRecordTemplateAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &WeightedDnsRecordTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedDnsRecordTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedDnsRecordTemplateAdjuster for TieredWeightedDnsRecordTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedDnsRecordTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted dns record template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedDnsRecordTemplateEngine {
    pub steps: Vec<Rc<dyn WeightedDnsRecordTemplateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedDnsRecordTemplateEngine {
    pub fn compute(&mut self, item: &WeightedDnsRecordTemplate) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[WeightedDnsRecordTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_dns_record_template_engine() -> WeightedDnsRecordTemplateEngine {
    WeightedDnsRecordTemplateEngine {
        steps: vec![
            Rc::new(MinimumWeightedDnsRecordTemplateAdjuster { minimum: 36.5 })
                as Rc<dyn WeightedDnsRecordTemplateAdjuster>,
            Rc::new(LocaleWeightedDnsRecordTemplateAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.78), ("south".to_string(), 1.05)]),
            }),
            Rc::new(TieredWeightedDnsRecordTemplateAdjuster {
                bounds: vec![4387.0, 8076.0],
                rates: vec![0.97, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 84685.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_dns_record_template_engine();
        let item = new_weighted_dns_record_template("code-597", "title-452");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
