use super::*;

use acme_kit::text;

/// Where a pending flow log notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFlowLogNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingFlowLogNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingFlowLogNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending flow log notice as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogNotice {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub currency: String,
    pub amount: f64,
    pub minimum: f64,
    pub reviewer: String,
    pub audience: String,
    pub pinned: bool,
    pub status: PendingFlowLogNoticeStatus,
}

impl PendingFlowLogNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFlowLogNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFlowLogNoticeStatus) -> bool {
        match self.status {
            PendingFlowLogNoticeStatus::Proposed => {
                return next == PendingFlowLogNoticeStatus::Approved;
            }
            PendingFlowLogNoticeStatus::Approved => {
                return next == PendingFlowLogNoticeStatus::Withdrawn;
            }
            PendingFlowLogNoticeStatus::Rejected => {
                match next {
                    PendingFlowLogNoticeStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingFlowLogNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending flow log notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.minimum + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingFlowLogNotice {
        PendingFlowLogNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            currency: self.currency.clone(),
            amount: self.amount,
            minimum: self.minimum,
            reviewer: self.reviewer.clone(),
            audience: self.audience.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending flow log notice in status proposed.
pub fn new_pending_flow_log_notice(code: &str, title: &str) -> PendingFlowLogNotice {
    PendingFlowLogNotice {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        currency: "".to_string(),
        amount: 0.0,
        minimum: 0.0,
        reviewer: "".to_string(),
        audience: "".to_string(),
        pinned: false,
        status: PendingFlowLogNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_flow_log_notice("code-420", "title-103");
        assert_eq!(item.status, PendingFlowLogNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_pending_flow_log_notice("code-420", "title-103");
        if item.move_to(PendingFlowLogNoticeStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingFlowLogNoticeStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_flow_log_notice("code-420", "title-103");
        item.status = PendingFlowLogNoticeStatus::Withdrawn;
        assert!(
            item.move_to(PendingFlowLogNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
