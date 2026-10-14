use super::*;

use acme_kit::text;

/// Where a silence digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SilenceDigestStatus {
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

impl std::fmt::Display for SilenceDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One silence digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceDigest {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub scope: String,
    pub ceiling: f64,
    pub status: SilenceDigestStatus,
}

impl SilenceDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceDigestStatus) -> bool {
        match self.status {
            SilenceDigestStatus::Idle => {
                match next {
                    SilenceDigestStatus::Draining | SilenceDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceDigestStatus::Warming => {
                match next {
                    SilenceDigestStatus::Ready | SilenceDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceDigestStatus::Ready => {
                match next {
                    SilenceDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceDigestStatus::Draining => {
                return next == SilenceDigestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "silence digest", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ceiling + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceDigest {
        SilenceDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            scope: self.scope.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a silence digest in status idle.
pub fn new_silence_digest(code: &str, title: &str) -> SilenceDigest {
    SilenceDigest {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        scope: "".to_string(),
        ceiling: 0.0,
        status: SilenceDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_silence_digest("code-390", "title-489");
        assert_eq!(item.status, SilenceDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_silence_digest("code-390", "title-489");
        if item.move_to(SilenceDigestStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceDigestStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_silence_digest("code-390", "title-489");
        item.status = SilenceDigestStatus::Stopped;
        assert!(
            item.move_to(SilenceDigestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
