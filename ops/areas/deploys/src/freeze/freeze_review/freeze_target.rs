use super::*;

use acme_kit::text;

/// Where a freeze target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FreezeTargetStatus {
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

impl std::fmt::Display for FreezeTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a freeze target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTarget {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub expires_at: i64,
    pub label: String,
    pub threshold: f64,
    pub status: FreezeTargetStatus,
}

impl FreezeTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeTargetStatus) -> bool {
        match self.status {
            FreezeTargetStatus::New => {
                match next {
                    FreezeTargetStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeTargetStatus::Queued => {
                return next == FreezeTargetStatus::Running || next == FreezeTargetStatus::Failed;
            }
            FreezeTargetStatus::Running => {
                match next {
                    FreezeTargetStatus::Done | FreezeTargetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeTargetStatus::Done => {
                match next {
                    FreezeTargetStatus::Failed => {
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

    pub fn move_to(&mut self, next: FreezeTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.expires_at as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeTarget {
        FreezeTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            expires_at: self.expires_at,
            label: self.label.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a freeze target in status new.
pub fn new_freeze_target(code: &str, name: &str) -> FreezeTarget {
    FreezeTarget {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        expires_at: 0,
        label: "".to_string(),
        threshold: 0.0,
        status: FreezeTargetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_freeze_target("code-794", "name-253");
        assert_eq!(item.status, FreezeTargetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_freeze_target("code-794", "name-253");
        if item.move_to(FreezeTargetStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeTargetStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_freeze_target("code-794", "name-253");
        item.status = FreezeTargetStatus::Failed;
        assert!(item.move_to(FreezeTargetStatus::New).is_err(), "expected move to fail");
    }
}
