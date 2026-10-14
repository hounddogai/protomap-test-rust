use super::*;

use acme_kit::text;

/// Where a slo digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SloDigestStatus {
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

impl std::fmt::Display for SloDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo digest as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloDigest {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub archived: bool,
    pub active: bool,
    pub margin: f64,
    pub status: SloDigestStatus,
}

impl SloDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloDigestStatus) -> bool {
        match self.status {
            SloDigestStatus::Idle => {
                match next {
                    SloDigestStatus::Ready | SloDigestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloDigestStatus::Warming => {
                match next {
                    SloDigestStatus::Ready | SloDigestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloDigestStatus::Ready => {
                return next == SloDigestStatus::Stopped;
            }
            SloDigestStatus::Draining => {
                match next {
                    SloDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SloDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> SloDigest {
        SloDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            archived: self.archived,
            active: self.active,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a slo digest in status idle.
pub fn new_slo_digest(code: &str, label: &str) -> SloDigest {
    SloDigest {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        archived: false,
        active: false,
        margin: 0.0,
        status: SloDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_slo_digest("code-730", "label-596");
        assert_eq!(item.status, SloDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_slo_digest("code-730", "label-596");
        if item.move_to(SloDigestStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloDigestStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_slo_digest("code-730", "label-596");
        item.status = SloDigestStatus::Stopped;
        assert!(item.move_to(SloDigestStatus::Idle).is_err(), "expected move to fail");
    }
}
