use super::*;

use acme_kit::text;

/// Where a tiered probe agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredProbeAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredProbeAgreementStatus {
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

impl std::fmt::Display for TieredProbeAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered probe agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeAgreement {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub priority: i64,
    pub duration_minutes: i64,
    pub archived: bool,
    pub notes: String,
    pub summary: String,
    pub status: TieredProbeAgreementStatus,
}

impl TieredProbeAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredProbeAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredProbeAgreementStatus) -> bool {
        match self.status {
            TieredProbeAgreementStatus::Draft => {
                return next == TieredProbeAgreementStatus::Pending || next == TieredProbeAgreementStatus::Suspended;
            }
            TieredProbeAgreementStatus::Pending => {
                match next {
                    TieredProbeAgreementStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredProbeAgreementStatus::Active => {
                match next {
                    TieredProbeAgreementStatus::Suspended | TieredProbeAgreementStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredProbeAgreementStatus::Suspended => {
                return next == TieredProbeAgreementStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredProbeAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered probe agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredProbeAgreement {
        TieredProbeAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            priority: self.priority,
            duration_minutes: self.duration_minutes,
            archived: self.archived,
            notes: self.notes.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a tiered probe agreement in status draft.
pub fn new_tiered_probe_agreement(code: &str, name: &str) -> TieredProbeAgreement {
    TieredProbeAgreement {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        priority: 0,
        duration_minutes: 0,
        archived: false,
        notes: "".to_string(),
        summary: "".to_string(),
        status: TieredProbeAgreementStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_probe_agreement("code-594", "name-854");
        assert_eq!(item.status, TieredProbeAgreementStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_tiered_probe_agreement("code-594", "name-854");
        if item.move_to(TieredProbeAgreementStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredProbeAgreementStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_probe_agreement("code-594", "name-854");
        item.status = TieredProbeAgreementStatus::Closed;
        assert!(
            item.move_to(TieredProbeAgreementStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
