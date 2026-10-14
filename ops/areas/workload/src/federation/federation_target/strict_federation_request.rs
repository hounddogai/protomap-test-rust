use super::*;

use acme_kit::text;

/// Where a strict federation request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictFederationRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictFederationRequestStatus {
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

impl std::fmt::Display for StrictFederationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict federation request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationRequest {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub version: i64,
    pub reviewer: String,
    pub status: StrictFederationRequestStatus,
}

impl StrictFederationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictFederationRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictFederationRequestStatus) -> bool {
        match self.status {
            StrictFederationRequestStatus::Draft => {
                match next {
                    StrictFederationRequestStatus::Suspended | StrictFederationRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictFederationRequestStatus::Pending => {
                return next == StrictFederationRequestStatus::Closed;
            }
            StrictFederationRequestStatus::Active => {
                match next {
                    StrictFederationRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictFederationRequestStatus::Suspended => {
                return next == StrictFederationRequestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictFederationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict federation request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.attempts as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictFederationRequest {
        StrictFederationRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            version: self.version,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict federation request in status draft.
pub fn new_strict_federation_request(code: &str, label: &str) -> StrictFederationRequest {
    StrictFederationRequest {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        version: 0,
        reviewer: "".to_string(),
        status: StrictFederationRequestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_federation_request("code-841", "label-22");
        assert_eq!(item.status, StrictFederationRequestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_strict_federation_request("code-841", "label-22");
        if item.move_to(StrictFederationRequestStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictFederationRequestStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_federation_request("code-841", "label-22");
        item.status = StrictFederationRequestStatus::Closed;
        assert!(
            item.move_to(StrictFederationRequestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
