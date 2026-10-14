use super::*;

use acme_kit::text;

/// Where a escalation manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationManifestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EscalationManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EscalationManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationManifest {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub source: String,
    pub checksum: String,
    pub rate: f64,
    pub quantity: i64,
    pub active: bool,
    pub status: EscalationManifestStatus,
}

impl EscalationManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationManifestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationManifestStatus) -> bool {
        match self.status {
            EscalationManifestStatus::Proposed => {
                return next == EscalationManifestStatus::Rejected || next == EscalationManifestStatus::Withdrawn;
            }
            EscalationManifestStatus::Approved => {
                return next == EscalationManifestStatus::Rejected;
            }
            EscalationManifestStatus::Rejected => {
                return next == EscalationManifestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "escalation manifest", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.quantity as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationManifest {
        EscalationManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            source: self.source.clone(),
            checksum: self.checksum.clone(),
            rate: self.rate,
            quantity: self.quantity,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a escalation manifest in status proposed.
pub fn new_escalation_manifest(code: &str, label: &str) -> EscalationManifest {
    EscalationManifest {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        source: "".to_string(),
        checksum: "".to_string(),
        rate: 0.0,
        quantity: 0,
        active: false,
        status: EscalationManifestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_escalation_manifest("code-509", "label-357");
        assert_eq!(item.status, EscalationManifestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_escalation_manifest("code-509", "label-357");
        if item.move_to(EscalationManifestStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationManifestStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_escalation_manifest("code-509", "label-357");
        item.status = EscalationManifestStatus::Withdrawn;
        assert!(
            item.move_to(EscalationManifestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
