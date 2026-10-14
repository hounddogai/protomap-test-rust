use super::*;

use acme_kit::text;

/// Where a capped quote metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedQuoteMetricStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedQuoteMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedQuoteMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped quote metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteMetric {
    pub code: String,
    pub title: String,
    pub description: String,
    pub weight: f64,
    pub currency: String,
    pub namespace: String,
    pub status: CappedQuoteMetricStatus,
}

impl CappedQuoteMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedQuoteMetricStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedQuoteMetricStatus) -> bool {
        match self.status {
            CappedQuoteMetricStatus::Proposed => {
                match next {
                    CappedQuoteMetricStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedQuoteMetricStatus::Approved => {
                match next {
                    CappedQuoteMetricStatus::Rejected | CappedQuoteMetricStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedQuoteMetricStatus::Rejected => {
                match next {
                    CappedQuoteMetricStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedQuoteMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped quote metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> CappedQuoteMetric {
        CappedQuoteMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            weight: self.weight,
            currency: self.currency.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a capped quote metric in status proposed.
pub fn new_capped_quote_metric(code: &str, title: &str) -> CappedQuoteMetric {
    CappedQuoteMetric {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        weight: 0.0,
        currency: "".to_string(),
        namespace: "".to_string(),
        status: CappedQuoteMetricStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_quote_metric("code-795", "title-489");
        assert_eq!(item.status, CappedQuoteMetricStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_capped_quote_metric("code-795", "title-489");
        if item.move_to(CappedQuoteMetricStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedQuoteMetricStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_quote_metric("code-795", "title-489");
        item.status = CappedQuoteMetricStatus::Withdrawn;
        assert!(
            item.move_to(CappedQuoteMetricStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
