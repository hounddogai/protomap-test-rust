use super::*;

use acme_kit::text;

/// Where a federation adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FederationAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FederationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation adjustment as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationAdjustment {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub created_at: i64,
    pub fingerprint: String,
    pub status: FederationAdjustmentStatus,
}

impl FederationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationAdjustmentStatus) -> bool {
        match self.status {
            FederationAdjustmentStatus::Proposed => {
                return next == FederationAdjustmentStatus::Rejected || next == FederationAdjustmentStatus::Withdrawn;
            }
            FederationAdjustmentStatus::Approved => {
                match next {
                    FederationAdjustmentStatus::Rejected | FederationAdjustmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationAdjustmentStatus::Rejected => {
                return next == FederationAdjustmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation adjustment {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.limit as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationAdjustment {
        FederationAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            created_at: self.created_at,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a federation adjustment in status proposed.
pub fn new_federation_adjustment(code: &str, title: &str) -> FederationAdjustment {
    FederationAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        created_at: 0,
        fingerprint: "".to_string(),
        status: FederationAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_federation_adjustment("code-136", "title-142");
        assert_eq!(item.status, FederationAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_federation_adjustment("code-136", "title-142");
        if item.move_to(FederationAdjustmentStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationAdjustmentStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_federation_adjustment("code-136", "title-142");
        item.status = FederationAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(FederationAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
