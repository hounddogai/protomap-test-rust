use super::*;

use acme_kit::text;

/// Where a weighted approval request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedApprovalRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedApprovalRequestStatus {
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

impl std::fmt::Display for WeightedApprovalRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted approval request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedApprovalRequest {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub rotation_days: i64,
    pub window_days: i64,
    pub locale: String,
    pub limit: i64,
    pub status: WeightedApprovalRequestStatus,
}

impl WeightedApprovalRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedApprovalRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedApprovalRequestStatus) -> bool {
        match self.status {
            WeightedApprovalRequestStatus::Draft => {
                match next {
                    WeightedApprovalRequestStatus::Pending | WeightedApprovalRequestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedApprovalRequestStatus::Pending => {
                return next == WeightedApprovalRequestStatus::Suspended
                    || next == WeightedApprovalRequestStatus::Closed;
            }
            WeightedApprovalRequestStatus::Active => {
                match next {
                    WeightedApprovalRequestStatus::Suspended | WeightedApprovalRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedApprovalRequestStatus::Suspended => {
                match next {
                    WeightedApprovalRequestStatus::Closed => {
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

    pub fn move_to(&mut self, next: WeightedApprovalRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted approval request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.sequence as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedApprovalRequest {
        WeightedApprovalRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            rotation_days: self.rotation_days,
            window_days: self.window_days,
            locale: self.locale.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a weighted approval request in status draft.
pub fn new_weighted_approval_request(code: &str, name: &str) -> WeightedApprovalRequest {
    WeightedApprovalRequest {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        rotation_days: 0,
        window_days: 0,
        locale: "".to_string(),
        limit: 0,
        status: WeightedApprovalRequestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_weighted_approval_request("code-755", "name-63");
        assert_eq!(item.status, WeightedApprovalRequestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_weighted_approval_request("code-755", "name-63");
        if item.move_to(WeightedApprovalRequestStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedApprovalRequestStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_weighted_approval_request("code-755", "name-63");
        item.status = WeightedApprovalRequestStatus::Closed;
        assert!(
            item.move_to(WeightedApprovalRequestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
