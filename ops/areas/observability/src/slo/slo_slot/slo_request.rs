use super::*;

use acme_kit::text;

/// Where a slo request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloRequestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SloRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SloRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a slo request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SloRequest {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub created_at: i64,
    pub updated_at: i64,
    pub sequence: i64,
    pub status: SloRequestStatus,
}

impl SloRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloRequestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloRequestStatus) -> bool {
        match self.status {
            SloRequestStatus::Proposed => {
                return next == SloRequestStatus::Approved || next == SloRequestStatus::Withdrawn;
            }
            SloRequestStatus::Approved => {
                return next == SloRequestStatus::Rejected;
            }
            SloRequestStatus::Rejected => {
                return next == SloRequestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "slo request", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.created_at as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloRequest {
        SloRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            created_at: self.created_at,
            updated_at: self.updated_at,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a slo request in status proposed.
pub fn new_slo_request(code: &str, label: &str) -> SloRequest {
    SloRequest {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        created_at: 0,
        updated_at: 0,
        sequence: 0,
        status: SloRequestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_slo_request("code-755", "label-418");
        assert_eq!(item.status, SloRequestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_slo_request("code-755", "label-418");
        if item.move_to(SloRequestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloRequestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_slo_request("code-755", "label-418");
        item.status = SloRequestStatus::Withdrawn;
        assert!(
            item.move_to(SloRequestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
