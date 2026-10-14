use super::*;

use acme_kit::text;

/// Where a freeze tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeTierStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FreezeTierStatus {
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

impl std::fmt::Display for FreezeTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One freeze tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTier {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub notes: String,
    pub window_days: i64,
    pub margin: f64,
    pub budget: f64,
    pub checksum: String,
    pub status: FreezeTierStatus,
}

impl FreezeTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeTierStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeTierStatus) -> bool {
        match self.status {
            FreezeTierStatus::New => {
                return next == FreezeTierStatus::Running || next == FreezeTierStatus::Done;
            }
            FreezeTierStatus::Queued => {
                match next {
                    FreezeTierStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeTierStatus::Running => {
                return next == FreezeTierStatus::Done;
            }
            FreezeTierStatus::Done => {
                return next == FreezeTierStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "freeze tier", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.rotation_days as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeTier {
        FreezeTier {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            notes: self.notes.clone(),
            window_days: self.window_days,
            margin: self.margin,
            budget: self.budget,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a freeze tier in status new.
pub fn new_freeze_tier(code: &str, title: &str) -> FreezeTier {
    FreezeTier {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        notes: "".to_string(),
        window_days: 0,
        margin: 0.0,
        budget: 0.0,
        checksum: "".to_string(),
        status: FreezeTierStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_freeze_tier("code-834", "title-120");
        assert_eq!(item.status, FreezeTierStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_freeze_tier("code-834", "title-120");
        if item.move_to(FreezeTierStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeTierStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_freeze_tier("code-834", "title-120");
        item.status = FreezeTierStatus::Failed;
        assert!(item.move_to(FreezeTierStatus::New).is_err(), "expected move to fail");
    }
}
