use super::*;

use acme_kit::text;

/// Where a lenient timeline rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTimelineRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientTimelineRollupStatus {
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

impl std::fmt::Display for LenientTimelineRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient timeline rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineRollup {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub expires_at: i64,
    pub sequence: i64,
    pub active: bool,
    pub floor_value: f64,
    pub principal: String,
    pub status: LenientTimelineRollupStatus,
}

impl LenientTimelineRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTimelineRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTimelineRollupStatus) -> bool {
        match self.status {
            LenientTimelineRollupStatus::Draft => {
                match next {
                    LenientTimelineRollupStatus::Pending | LenientTimelineRollupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTimelineRollupStatus::Pending => {
                return next == LenientTimelineRollupStatus::Closed;
            }
            LenientTimelineRollupStatus::Active => {
                return next == LenientTimelineRollupStatus::Suspended;
            }
            LenientTimelineRollupStatus::Suspended => {
                return next == LenientTimelineRollupStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientTimelineRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient timeline rollup {} cannot move from {} to {}",
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
        format!("{} {}", "lenient timeline rollup", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.sequence as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTimelineRollup {
        LenientTimelineRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            expires_at: self.expires_at,
            sequence: self.sequence,
            active: self.active,
            floor_value: self.floor_value,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient timeline rollup in status draft.
pub fn new_lenient_timeline_rollup(code: &str, label: &str) -> LenientTimelineRollup {
    LenientTimelineRollup {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        expires_at: 0,
        sequence: 0,
        active: false,
        floor_value: 0.0,
        principal: "".to_string(),
        status: LenientTimelineRollupStatus::Draft,
    }
}
