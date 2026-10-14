use super::*;

use acme_kit::text;

/// Where a pending health check preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingHealthCheckPreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingHealthCheckPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingHealthCheckPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending health check preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckPreference {
    pub code: String,
    pub name: String,
    pub region: String,
    pub audience: String,
    pub pinned: bool,
    pub contact_email: String,
    pub updated_at: i64,
    pub checksum: String,
    pub cluster: String,
    pub burst: i64,
    pub status: PendingHealthCheckPreferenceStatus,
}

impl PendingHealthCheckPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingHealthCheckPreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingHealthCheckPreferenceStatus) -> bool {
        match self.status {
            PendingHealthCheckPreferenceStatus::Proposed => {
                match next {
                    PendingHealthCheckPreferenceStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingHealthCheckPreferenceStatus::Approved => {
                return next == PendingHealthCheckPreferenceStatus::Rejected
                    || next == PendingHealthCheckPreferenceStatus::Withdrawn;
            }
            PendingHealthCheckPreferenceStatus::Rejected => {
                match next {
                    PendingHealthCheckPreferenceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingHealthCheckPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending health check preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.burst as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingHealthCheckPreference {
        PendingHealthCheckPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            audience: self.audience.clone(),
            pinned: self.pinned,
            contact_email: self.contact_email.clone(),
            updated_at: self.updated_at,
            checksum: self.checksum.clone(),
            cluster: self.cluster.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a pending health check preference in status proposed.
pub fn new_pending_health_check_preference(code: &str, name: &str) -> PendingHealthCheckPreference {
    PendingHealthCheckPreference {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        audience: "".to_string(),
        pinned: false,
        contact_email: "".to_string(),
        updated_at: 0,
        checksum: "".to_string(),
        cluster: "".to_string(),
        burst: 0,
        status: PendingHealthCheckPreferenceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_health_check_preference("code-252", "name-919");
        assert_eq!(item.status, PendingHealthCheckPreferenceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_pending_health_check_preference("code-252", "name-919");
        if item.move_to(PendingHealthCheckPreferenceStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingHealthCheckPreferenceStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_health_check_preference("code-252", "name-919");
        item.status = PendingHealthCheckPreferenceStatus::Withdrawn;
        assert!(
            item.move_to(PendingHealthCheckPreferenceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
