use super::*;

use acme_kit::text;

/// Where a federation request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FederationRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for FederationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation request as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationRequest {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub title: String,
    pub quantity: i64,
    pub status: FederationRequestStatus,
}

impl FederationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationRequestStatus) -> bool {
        match self.status {
            FederationRequestStatus::New => {
                match next {
                    FederationRequestStatus::Running | FederationRequestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationRequestStatus::Queued => {
                match next {
                    FederationRequestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationRequestStatus::Running => {
                return next == FederationRequestStatus::Done;
            }
            FederationRequestStatus::Done => {
                match next {
                    FederationRequestStatus::Failed => {
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

    pub fn move_to(&mut self, next: FederationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "federation request", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.quota_units as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationRequest {
        FederationRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            title: self.title.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a federation request in status new.
pub fn new_federation_request(code: &str, name: &str) -> FederationRequest {
    FederationRequest {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        title: "".to_string(),
        quantity: 0,
        status: FederationRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_federation_request("code-318", "name-329");
        assert_eq!(item.status, FederationRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_federation_request("code-318", "name-329");
        if item.move_to(FederationRequestStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationRequestStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_federation_request("code-318", "name-329");
        item.status = FederationRequestStatus::Failed;
        assert!(
            item.move_to(FederationRequestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
