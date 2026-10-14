use super::*;

use acme_kit::text;

/// Where a lenient dashboard summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientDashboardSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientDashboardSummaryStatus {
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

impl std::fmt::Display for LenientDashboardSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient dashboard summary as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardSummary {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub external_ref: String,
    pub cipher: String,
    pub unit: String,
    pub maximum: f64,
    pub currency: String,
    pub quantity: i64,
    pub status: LenientDashboardSummaryStatus,
}

impl LenientDashboardSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientDashboardSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientDashboardSummaryStatus) -> bool {
        match self.status {
            LenientDashboardSummaryStatus::Draft => {
                return next == LenientDashboardSummaryStatus::Suspended;
            }
            LenientDashboardSummaryStatus::Pending => {
                return next == LenientDashboardSummaryStatus::Suspended;
            }
            LenientDashboardSummaryStatus::Active => {
                return next == LenientDashboardSummaryStatus::Suspended
                    || next == LenientDashboardSummaryStatus::Closed;
            }
            LenientDashboardSummaryStatus::Suspended => {
                match next {
                    LenientDashboardSummaryStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientDashboardSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient dashboard summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient dashboard summary", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.maximum + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientDashboardSummary {
        LenientDashboardSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            external_ref: self.external_ref.clone(),
            cipher: self.cipher.clone(),
            unit: self.unit.clone(),
            maximum: self.maximum,
            currency: self.currency.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient dashboard summary in status draft.
pub fn new_lenient_dashboard_summary(code: &str, title: &str) -> LenientDashboardSummary {
    LenientDashboardSummary {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        external_ref: "".to_string(),
        cipher: "".to_string(),
        unit: "".to_string(),
        maximum: 0.0,
        currency: "".to_string(),
        quantity: 0,
        status: LenientDashboardSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_dashboard_summary("code-148", "title-38");
        assert_eq!(item.status, LenientDashboardSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_lenient_dashboard_summary("code-148", "title-38");
        if item.move_to(LenientDashboardSummaryStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientDashboardSummaryStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_dashboard_summary("code-148", "title-38");
        item.status = LenientDashboardSummaryStatus::Closed;
        assert!(
            item.move_to(LenientDashboardSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
