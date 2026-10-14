use super::*;

use acme_kit::text;

/// Where a rolling alert proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingAlertProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingAlertProposalStatus {
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

impl std::fmt::Display for RollingAlertProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling alert proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertProposal {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub timezone: String,
    pub region: String,
    pub phone: String,
    pub category: String,
    pub archived: bool,
    pub priority: i64,
    pub status: RollingAlertProposalStatus,
}

impl RollingAlertProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingAlertProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingAlertProposalStatus) -> bool {
        match self.status {
            RollingAlertProposalStatus::Draft => {
                return next == RollingAlertProposalStatus::Suspended;
            }
            RollingAlertProposalStatus::Pending => {
                return next == RollingAlertProposalStatus::Closed;
            }
            RollingAlertProposalStatus::Active => {
                return next == RollingAlertProposalStatus::Suspended;
            }
            RollingAlertProposalStatus::Suspended => {
                match next {
                    RollingAlertProposalStatus::Closed => {
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

    pub fn move_to(&mut self, next: RollingAlertProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling alert proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.priority as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingAlertProposal {
        RollingAlertProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            timezone: self.timezone.clone(),
            region: self.region.clone(),
            phone: self.phone.clone(),
            category: self.category.clone(),
            archived: self.archived,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rolling alert proposal in status draft.
pub fn new_rolling_alert_proposal(code: &str, label: &str) -> RollingAlertProposal {
    RollingAlertProposal {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        timezone: "".to_string(),
        region: "".to_string(),
        phone: "".to_string(),
        category: "".to_string(),
        archived: false,
        priority: 0,
        status: RollingAlertProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_alert_proposal("code-718", "label-890");
        assert_eq!(item.status, RollingAlertProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_rolling_alert_proposal("code-718", "label-890");
        if item.move_to(RollingAlertProposalStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingAlertProposalStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_alert_proposal("code-718", "label-890");
        item.status = RollingAlertProposalStatus::Closed;
        assert!(
            item.move_to(RollingAlertProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
