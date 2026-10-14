use super::*;

use acme_kit::text;

/// Where a evidence snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EvidenceSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for EvidenceSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One evidence snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSnapshot {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub grace_days: i64,
    pub score: f64,
    pub status: EvidenceSnapshotStatus,
}

impl EvidenceSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceSnapshotStatus) -> bool {
        match self.status {
            EvidenceSnapshotStatus::Planned => {
                return next == EvidenceSnapshotStatus::Scheduled || next == EvidenceSnapshotStatus::InProgress;
            }
            EvidenceSnapshotStatus::Scheduled => {
                return next == EvidenceSnapshotStatus::InProgress || next == EvidenceSnapshotStatus::Cancelled;
            }
            EvidenceSnapshotStatus::InProgress => {
                match next {
                    EvidenceSnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceSnapshotStatus::Completed => {
                match next {
                    EvidenceSnapshotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EvidenceSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.grace_days as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceSnapshot {
        EvidenceSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            grace_days: self.grace_days,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a evidence snapshot in status planned.
pub fn new_evidence_snapshot(code: &str, title: &str) -> EvidenceSnapshot {
    EvidenceSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        grace_days: 0,
        score: 0.0,
        status: EvidenceSnapshotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_evidence_snapshot("code-289", "title-701");
        assert_eq!(item.status, EvidenceSnapshotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_evidence_snapshot("code-289", "title-701");
        if item.move_to(EvidenceSnapshotStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceSnapshotStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_evidence_snapshot("code-289", "title-701");
        item.status = EvidenceSnapshotStatus::Cancelled;
        assert!(
            item.move_to(EvidenceSnapshotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
