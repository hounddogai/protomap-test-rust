use super::*;

use acme_kit::text;

/// Where a egress cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EgressCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EgressCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress cycle as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressCycle {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub checksum: String,
    pub source: String,
    pub duration_minutes: i64,
    pub external_ref: String,
    pub threshold: f64,
    pub cipher: String,
    pub phone: String,
    pub starts_at: i64,
    pub status: EgressCycleStatus,
}

impl EgressCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressCycleStatus) -> bool {
        match self.status {
            EgressCycleStatus::Proposed => {
                return next == EgressCycleStatus::Rejected;
            }
            EgressCycleStatus::Approved => {
                return next == EgressCycleStatus::Rejected;
            }
            EgressCycleStatus::Rejected => {
                match next {
                    EgressCycleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: EgressCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.attempts as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressCycle {
        EgressCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            checksum: self.checksum.clone(),
            source: self.source.clone(),
            duration_minutes: self.duration_minutes,
            external_ref: self.external_ref.clone(),
            threshold: self.threshold,
            cipher: self.cipher.clone(),
            phone: self.phone.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a egress cycle in status proposed.
pub fn new_egress_cycle(code: &str, label: &str) -> EgressCycle {
    EgressCycle {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        checksum: "".to_string(),
        source: "".to_string(),
        duration_minutes: 0,
        external_ref: "".to_string(),
        threshold: 0.0,
        cipher: "".to_string(),
        phone: "".to_string(),
        starts_at: 0,
        status: EgressCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_egress_cycle("code-940", "label-333");
        assert_eq!(item.status, EgressCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_egress_cycle("code-940", "label-333");
        if item.move_to(EgressCycleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressCycleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_egress_cycle("code-940", "label-333");
        item.status = EgressCycleStatus::Withdrawn;
        assert!(
            item.move_to(EgressCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
