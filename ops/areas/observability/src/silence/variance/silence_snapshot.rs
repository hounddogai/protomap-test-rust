use super::*;

use acme_kit::text;

/// Where a silence snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SilenceSnapshotStatus {
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

impl std::fmt::Display for SilenceSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSnapshot {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub cipher: String,
    pub principal: String,
    pub amount: f64,
    pub status: SilenceSnapshotStatus,
}

impl SilenceSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceSnapshotStatus) -> bool {
        match self.status {
            SilenceSnapshotStatus::New => {
                match next {
                    SilenceSnapshotStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceSnapshotStatus::Queued => {
                return next == SilenceSnapshotStatus::Failed;
            }
            SilenceSnapshotStatus::Running => {
                return next == SilenceSnapshotStatus::Done || next == SilenceSnapshotStatus::Failed;
            }
            SilenceSnapshotStatus::Done => {
                match next {
                    SilenceSnapshotStatus::Failed => {
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

    pub fn move_to(&mut self, next: SilenceSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "silence snapshot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> SilenceSnapshot {
        SilenceSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            cipher: self.cipher.clone(),
            principal: self.principal.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a silence snapshot in status new.
pub fn new_silence_snapshot(code: &str, label: &str) -> SilenceSnapshot {
    SilenceSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        cipher: "".to_string(),
        principal: "".to_string(),
        amount: 0.0,
        status: SilenceSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_silence_snapshot("code-270", "label-92");
        assert_eq!(item.status, SilenceSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_silence_snapshot("code-270", "label-92");
        if item.move_to(SilenceSnapshotStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceSnapshotStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_silence_snapshot("code-270", "label-92");
        item.status = SilenceSnapshotStatus::Failed;
        assert!(
            item.move_to(SilenceSnapshotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
