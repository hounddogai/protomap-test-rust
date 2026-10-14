use super::*;

use acme_kit::text;

/// Where a lenient sink draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSinkDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientSinkDraftStatus {
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

impl std::fmt::Display for LenientSinkDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient sink draft as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkDraft {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub ratio: f64,
    pub minimum: f64,
    pub summary: String,
    pub source: String,
    pub checksum: String,
    pub status: LenientSinkDraftStatus,
}

impl LenientSinkDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSinkDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSinkDraftStatus) -> bool {
        match self.status {
            LenientSinkDraftStatus::Idle => {
                return next == LenientSinkDraftStatus::Draining;
            }
            LenientSinkDraftStatus::Warming => {
                return next == LenientSinkDraftStatus::Ready || next == LenientSinkDraftStatus::Stopped;
            }
            LenientSinkDraftStatus::Ready => {
                match next {
                    LenientSinkDraftStatus::Draining | LenientSinkDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSinkDraftStatus::Draining => {
                return next == LenientSinkDraftStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientSinkDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient sink draft {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "lenient sink draft", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.expires_at as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSinkDraft {
        LenientSinkDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            ratio: self.ratio,
            minimum: self.minimum,
            summary: self.summary.clone(),
            source: self.source.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a lenient sink draft in status idle.
pub fn new_lenient_sink_draft(code: &str, title: &str) -> LenientSinkDraft {
    LenientSinkDraft {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        ratio: 0.0,
        minimum: 0.0,
        summary: "".to_string(),
        source: "".to_string(),
        checksum: "".to_string(),
        status: LenientSinkDraftStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_sink_draft("code-839", "title-256");
        assert_eq!(item.status, LenientSinkDraftStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_lenient_sink_draft("code-839", "title-256");
        if item.move_to(LenientSinkDraftStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSinkDraftStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_sink_draft("code-839", "title-256");
        item.status = LenientSinkDraftStatus::Stopped;
        assert!(
            item.move_to(LenientSinkDraftStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
