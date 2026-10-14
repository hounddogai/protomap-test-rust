use super::*;

use acme_kit::text;

/// Where a regional quote metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalQuoteMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalQuoteMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RegionalQuoteMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional quote metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteMetric {
    pub code: String,
    pub title: String,
    pub description: String,
    pub issuer: String,
    pub phone: String,
    pub score: f64,
    pub label: String,
    pub status: RegionalQuoteMetricStatus,
}

impl RegionalQuoteMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalQuoteMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalQuoteMetricStatus) -> bool {
        match self.status {
            RegionalQuoteMetricStatus::Draft => {
                return next == RegionalQuoteMetricStatus::Pending;
            }
            RegionalQuoteMetricStatus::Pending => {
                return next == RegionalQuoteMetricStatus::Suspended;
            }
            RegionalQuoteMetricStatus::Active => {
                return next == RegionalQuoteMetricStatus::Suspended || next == RegionalQuoteMetricStatus::Closed;
            }
            RegionalQuoteMetricStatus::Suspended => {
                return next == RegionalQuoteMetricStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalQuoteMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional quote metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> RegionalQuoteMetric {
        RegionalQuoteMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            issuer: self.issuer.clone(),
            phone: self.phone.clone(),
            score: self.score,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional quote metric in status draft.
pub fn new_regional_quote_metric(code: &str, title: &str) -> RegionalQuoteMetric {
    RegionalQuoteMetric {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        issuer: "".to_string(),
        phone: "".to_string(),
        score: 0.0,
        label: "".to_string(),
        status: RegionalQuoteMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_quote_metric("code-257", "title-923");
        assert_eq!(item.status, RegionalQuoteMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_regional_quote_metric("code-257", "title-923");
        if item.move_to(RegionalQuoteMetricStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalQuoteMetricStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_quote_metric("code-257", "title-923");
        item.status = RegionalQuoteMetricStatus::Closed;
        assert!(
            item.move_to(RegionalQuoteMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
