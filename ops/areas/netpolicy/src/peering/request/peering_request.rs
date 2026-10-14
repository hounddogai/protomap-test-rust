use super::*;

use acme_kit::text;

/// Where a peering request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringRequestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PeeringRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PeeringRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering request as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRequest {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub scope: String,
    pub unit: String,
    pub status: PeeringRequestStatus,
}

impl PeeringRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringRequestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringRequestStatus) -> bool {
        match self.status {
            PeeringRequestStatus::Open => {
                match next {
                    PeeringRequestStatus::InReview | PeeringRequestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringRequestStatus::InReview => {
                return next == PeeringRequestStatus::Resolved || next == PeeringRequestStatus::Reopened;
            }
            PeeringRequestStatus::Resolved => {
                return next == PeeringRequestStatus::Archived;
            }
            PeeringRequestStatus::Reopened => {
                match next {
                    PeeringRequestStatus::Archived => {
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

    pub fn move_to(&mut self, next: PeeringRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering request {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "peering request", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> PeeringRequest {
        PeeringRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            scope: self.scope.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a peering request in status open.
pub fn new_peering_request(code: &str, name: &str) -> PeeringRequest {
    PeeringRequest {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        scope: "".to_string(),
        unit: "".to_string(),
        status: PeeringRequestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_peering_request("code-753", "name-198");
        assert_eq!(item.status, PeeringRequestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_peering_request("code-753", "name-198");
        if item.move_to(PeeringRequestStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringRequestStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_peering_request("code-753", "name-198");
        item.status = PeeringRequestStatus::Archived;
        assert!(
            item.move_to(PeeringRequestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
