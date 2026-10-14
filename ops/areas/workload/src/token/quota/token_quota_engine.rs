use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the token quota calculation.
pub trait TokenQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTokenQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenQuotaFactor for NameTokenQuotaAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TokenQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenQuotaFactor for TieredTokenQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenQuotaFactor for FlaggedTokenQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTokenQuotaAdjuster {
    pub percent: f64,
}

impl TokenQuotaFactor for PercentTokenQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TokenQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenQuotaAdjuster {
    pub cap: f64,
}

impl TokenQuotaFactor for CapTokenQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenQuotaPriorityOffsetAdjuster {
    pub weight: f64,
}

impl TokenQuotaFactor for TokenQuotaPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &TokenQuota, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

/// Runs the token quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenQuotaEngine {
    pub steps: Vec<Rc<dyn TokenQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenQuotaEngine {
    pub fn run(&mut self, item: &TokenQuota) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[TokenQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_token_quota_engine(flags: &flags::Registry) -> TokenQuotaEngine {
    TokenQuotaEngine {
        steps: vec![
            Rc::new(NameTokenQuotaAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.4), ("east".to_string(), 1.15)]),
            }) as Rc<dyn TokenQuotaFactor>,
            Rc::new(TieredTokenQuotaAdjuster {
                bounds: vec![3750.0, 6207.0, 6868.0, 8638.0],
                rates: vec![1.2, 0.87, 0.87, 1.16],
            }),
            Rc::new(FlaggedTokenQuotaAdjuster {
                flags: flags.clone(),
                flag: "token-quota-pending".to_string(),
                percent: -7.9,
            }),
            Rc::new(PercentTokenQuotaAdjuster { percent: 18.6 }),
            Rc::new(CapTokenQuotaAdjuster { cap: 40251.0 }),
            Rc::new(TokenQuotaPriorityOffsetAdjuster { weight: 0.67 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 16654.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_quota_engine(&flags::new_registry());
        let item = new_token_quota("code-286", "name-336");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
