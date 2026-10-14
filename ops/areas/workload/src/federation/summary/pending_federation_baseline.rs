use super::*;

use acme_kit::text;

/// Where a pending federation baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFederationBaselineStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingFederationBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingFederationBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending federation baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationBaseline {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub host: String,
    pub fingerprint: String,
    pub label: String,
    pub audience: String,
    pub ratio: f64,
    pub version: i64,
    pub status: PendingFederationBaselineStatus,
}

impl PendingFederationBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFederationBaselineStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFederationBaselineStatus) -> bool {
        match self.status {
            PendingFederationBaselineStatus::Proposed => {
                match next {
                    PendingFederationBaselineStatus::Rejected | PendingFederationBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFederationBaselineStatus::Approved => {
                match next {
                    PendingFederationBaselineStatus::Rejected | PendingFederationBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFederationBaselineStatus::Rejected => {
                match next {
                    PendingFederationBaselineStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingFederationBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending federation baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.version as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingFederationBaseline {
        PendingFederationBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            host: self.host.clone(),
            fingerprint: self.fingerprint.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            ratio: self.ratio,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending federation baseline in status proposed.
pub fn new_pending_federation_baseline(code: &str, name: &str) -> PendingFederationBaseline {
    PendingFederationBaseline {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        host: "".to_string(),
        fingerprint: "".to_string(),
        label: "".to_string(),
        audience: "".to_string(),
        ratio: 0.0,
        version: 0,
        status: PendingFederationBaselineStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_federation_baseline("code-289", "name-534");
        assert_eq!(item.status, PendingFederationBaselineStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_pending_federation_baseline("code-289", "name-534");
        if item.move_to(PendingFederationBaselineStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingFederationBaselineStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_federation_baseline("code-289", "name-534");
        item.status = PendingFederationBaselineStatus::Withdrawn;
        assert!(
            item.move_to(PendingFederationBaselineStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
