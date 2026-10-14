use super::*;

use acme_kit::text;

/// Where a approval summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ApprovalSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ApprovalSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a approval summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSummary {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub quantity: i64,
    pub burst: i64,
    pub country: String,
    pub phone: String,
    pub priority: i64,
    pub status: ApprovalSummaryStatus,
}

impl ApprovalSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalSummaryStatus) -> bool {
        match self.status {
            ApprovalSummaryStatus::Requested => {
                return next == ApprovalSummaryStatus::Granted || next == ApprovalSummaryStatus::Expired;
            }
            ApprovalSummaryStatus::Granted => {
                return next == ApprovalSummaryStatus::Revoked || next == ApprovalSummaryStatus::Expired;
            }
            ApprovalSummaryStatus::Revoked => {
                return next == ApprovalSummaryStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "approval summary", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.maximum - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalSummary {
        ApprovalSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            quantity: self.quantity,
            burst: self.burst,
            country: self.country.clone(),
            phone: self.phone.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a approval summary in status requested.
pub fn new_approval_summary(code: &str, title: &str) -> ApprovalSummary {
    ApprovalSummary {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        quantity: 0,
        burst: 0,
        country: "".to_string(),
        phone: "".to_string(),
        priority: 0,
        status: ApprovalSummaryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_approval_summary("code-1", "title-910");
        assert_eq!(item.status, ApprovalSummaryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_approval_summary("code-1", "title-910");
        if item.move_to(ApprovalSummaryStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalSummaryStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_approval_summary("code-1", "title-910");
        item.status = ApprovalSummaryStatus::Expired;
        assert!(
            item.move_to(ApprovalSummaryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
