use super::*;

use acme_kit::text;

/// Where a strict allowlist bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictAllowlistBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictAllowlistBundleStatus {
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

impl std::fmt::Display for StrictAllowlistBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict allowlist bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistBundle {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub burst: i64,
    pub locale: String,
    pub retries: i64,
    pub contact_email: String,
    pub status: StrictAllowlistBundleStatus,
}

impl StrictAllowlistBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictAllowlistBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictAllowlistBundleStatus) -> bool {
        match self.status {
            StrictAllowlistBundleStatus::Idle => {
                return next == StrictAllowlistBundleStatus::Ready;
            }
            StrictAllowlistBundleStatus::Warming => {
                match next {
                    StrictAllowlistBundleStatus::Ready | StrictAllowlistBundleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAllowlistBundleStatus::Ready => {
                return next == StrictAllowlistBundleStatus::Draining;
            }
            StrictAllowlistBundleStatus::Draining => {
                match next {
                    StrictAllowlistBundleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: StrictAllowlistBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict allowlist bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.retries as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictAllowlistBundle {
        StrictAllowlistBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            burst: self.burst,
            locale: self.locale.clone(),
            retries: self.retries,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a strict allowlist bundle in status idle.
pub fn new_strict_allowlist_bundle(code: &str, title: &str) -> StrictAllowlistBundle {
    StrictAllowlistBundle {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        burst: 0,
        locale: "".to_string(),
        retries: 0,
        contact_email: "".to_string(),
        status: StrictAllowlistBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_allowlist_bundle("code-949", "title-936");
        assert_eq!(item.status, StrictAllowlistBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_strict_allowlist_bundle("code-949", "title-936");
        if item.move_to(StrictAllowlistBundleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictAllowlistBundleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_allowlist_bundle("code-949", "title-936");
        item.status = StrictAllowlistBundleStatus::Stopped;
        assert!(
            item.move_to(StrictAllowlistBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
