use super::*;

use acme_kit::text;

/// Where a fallback freeze window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackFreezeWindowStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackFreezeWindowStatus {
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

impl std::fmt::Display for FallbackFreezeWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback freeze window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFreezeWindow {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub namespace: String,
    pub key_id: String,
    pub rotation_days: i64,
    pub version: i64,
    pub title: String,
    pub status: FallbackFreezeWindowStatus,
}

impl FallbackFreezeWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackFreezeWindowStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackFreezeWindowStatus) -> bool {
        match self.status {
            FallbackFreezeWindowStatus::New => {
                match next {
                    FallbackFreezeWindowStatus::Running | FallbackFreezeWindowStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFreezeWindowStatus::Queued => {
                match next {
                    FallbackFreezeWindowStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFreezeWindowStatus::Running => {
                match next {
                    FallbackFreezeWindowStatus::Done | FallbackFreezeWindowStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFreezeWindowStatus::Done => {
                match next {
                    FallbackFreezeWindowStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackFreezeWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback freeze window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.version as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackFreezeWindow {
        FallbackFreezeWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            namespace: self.namespace.clone(),
            key_id: self.key_id.clone(),
            rotation_days: self.rotation_days,
            version: self.version,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback freeze window in status new.
pub fn new_fallback_freeze_window(code: &str, name: &str) -> FallbackFreezeWindow {
    FallbackFreezeWindow {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        namespace: "".to_string(),
        key_id: "".to_string(),
        rotation_days: 0,
        version: 0,
        title: "".to_string(),
        status: FallbackFreezeWindowStatus::New,
    }
}
