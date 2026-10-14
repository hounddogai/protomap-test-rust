use super::*;

use acme_kit::text;

/// Where a strict probe snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictProbeSnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictProbeSnapshotStatus {
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

impl std::fmt::Display for StrictProbeSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict probe snapshot as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictProbeSnapshot {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub batch_size: i64,
    pub description: String,
    pub ratio: f64,
    pub currency: String,
    pub cipher: String,
    pub status: StrictProbeSnapshotStatus,
}

impl StrictProbeSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictProbeSnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictProbeSnapshotStatus) -> bool {
        match self.status {
            StrictProbeSnapshotStatus::Open => {
                match next {
                    StrictProbeSnapshotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictProbeSnapshotStatus::InReview => {
                match next {
                    StrictProbeSnapshotStatus::Resolved | StrictProbeSnapshotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictProbeSnapshotStatus::Resolved => {
                match next {
                    StrictProbeSnapshotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictProbeSnapshotStatus::Reopened => {
                match next {
                    StrictProbeSnapshotStatus::Archived => {
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

    pub fn move_to(&mut self, next: StrictProbeSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict probe snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.batch_size as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictProbeSnapshot {
        StrictProbeSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            batch_size: self.batch_size,
            description: self.description.clone(),
            ratio: self.ratio,
            currency: self.currency.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict probe snapshot in status open.
pub fn new_strict_probe_snapshot(code: &str, name: &str) -> StrictProbeSnapshot {
    StrictProbeSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        batch_size: 0,
        description: "".to_string(),
        ratio: 0.0,
        currency: "".to_string(),
        cipher: "".to_string(),
        status: StrictProbeSnapshotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_probe_snapshot("code-63", "name-909");
        assert_eq!(item.status, StrictProbeSnapshotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_strict_probe_snapshot("code-63", "name-909");
        if item.move_to(StrictProbeSnapshotStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictProbeSnapshotStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_probe_snapshot("code-63", "name-909");
        item.status = StrictProbeSnapshotStatus::Archived;
        assert!(
            item.move_to(StrictProbeSnapshotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
