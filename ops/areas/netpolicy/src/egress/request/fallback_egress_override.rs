use super::*;

use acme_kit::text;

/// Where a fallback egress override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackEgressOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackEgressOverrideStatus {
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

impl std::fmt::Display for FallbackEgressOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback egress override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressOverride {
    pub code: String,
    pub name: String,
    pub source: String,
    pub cipher: String,
    pub version: i64,
    pub issuer: String,
    pub status: FallbackEgressOverrideStatus,
}

impl FallbackEgressOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackEgressOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackEgressOverrideStatus) -> bool {
        match self.status {
            FallbackEgressOverrideStatus::New => {
                return next == FallbackEgressOverrideStatus::Failed;
            }
            FallbackEgressOverrideStatus::Queued => {
                match next {
                    FallbackEgressOverrideStatus::Running | FallbackEgressOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEgressOverrideStatus::Running => {
                return next == FallbackEgressOverrideStatus::Failed;
            }
            FallbackEgressOverrideStatus::Done => {
                match next {
                    FallbackEgressOverrideStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackEgressOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback egress override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> FallbackEgressOverride {
        FallbackEgressOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            cipher: self.cipher.clone(),
            version: self.version,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a fallback egress override in status new.
pub fn new_fallback_egress_override(code: &str, name: &str) -> FallbackEgressOverride {
    FallbackEgressOverride {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        cipher: "".to_string(),
        version: 0,
        issuer: "".to_string(),
        status: FallbackEgressOverrideStatus::New,
    }
}
