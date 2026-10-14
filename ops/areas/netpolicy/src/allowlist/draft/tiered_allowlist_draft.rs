use super::*;

use acme_kit::text;

/// Where a tiered allowlist draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredAllowlistDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredAllowlistDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TieredAllowlistDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered allowlist draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistDraft {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub active: bool,
    pub channel: String,
    pub port: i64,
    pub status: TieredAllowlistDraftStatus,
}

impl TieredAllowlistDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredAllowlistDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredAllowlistDraftStatus) -> bool {
        match self.status {
            TieredAllowlistDraftStatus::Idle => {
                return next == TieredAllowlistDraftStatus::Draining;
            }
            TieredAllowlistDraftStatus::Warming => {
                return next == TieredAllowlistDraftStatus::Ready || next == TieredAllowlistDraftStatus::Stopped;
            }
            TieredAllowlistDraftStatus::Ready => {
                match next {
                    TieredAllowlistDraftStatus::Draining | TieredAllowlistDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredAllowlistDraftStatus::Draining => {
                return next == TieredAllowlistDraftStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredAllowlistDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered allowlist draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.port as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredAllowlistDraft {
        TieredAllowlistDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            active: self.active,
            channel: self.channel.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a tiered allowlist draft in status idle.
pub fn new_tiered_allowlist_draft(code: &str, name: &str) -> TieredAllowlistDraft {
    TieredAllowlistDraft {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        active: false,
        channel: "".to_string(),
        port: 0,
        status: TieredAllowlistDraftStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_allowlist_draft("code-173", "name-912");
        assert_eq!(item.status, TieredAllowlistDraftStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_tiered_allowlist_draft("code-173", "name-912");
        if item.move_to(TieredAllowlistDraftStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredAllowlistDraftStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_allowlist_draft("code-173", "name-912");
        item.status = TieredAllowlistDraftStatus::Stopped;
        assert!(
            item.move_to(TieredAllowlistDraftStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
