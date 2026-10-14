use super::*;

use acme_kit::text;

/// Where a alert request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertRequestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AlertRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AlertRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert request as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertRequest {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub owner_id: String,
    pub expires_at: i64,
    pub status: AlertRequestStatus,
}

impl AlertRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertRequestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertRequestStatus) -> bool {
        match self.status {
            AlertRequestStatus::Proposed => {
                match next {
                    AlertRequestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertRequestStatus::Approved => {
                return next == AlertRequestStatus::Withdrawn;
            }
            AlertRequestStatus::Rejected => {
                match next {
                    AlertRequestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AlertRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "alert request", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> AlertRequest {
        AlertRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            owner_id: self.owner_id.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a alert request in status proposed.
pub fn new_alert_request(code: &str, label: &str) -> AlertRequest {
    AlertRequest {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        owner_id: "".to_string(),
        expires_at: 0,
        status: AlertRequestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_alert_request("code-131", "label-724");
        assert_eq!(item.status, AlertRequestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_alert_request("code-131", "label-724");
        if item.move_to(AlertRequestStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertRequestStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_alert_request("code-131", "label-724");
        item.status = AlertRequestStatus::Withdrawn;
        assert!(
            item.move_to(AlertRequestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
