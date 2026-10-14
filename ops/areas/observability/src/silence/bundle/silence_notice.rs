use super::*;

use acme_kit::text;

/// Where a silence notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceNoticeStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SilenceNoticeStatus {
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

impl std::fmt::Display for SilenceNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a silence notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceNotice {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub active: bool,
    pub version: i64,
    pub status: SilenceNoticeStatus,
}

impl SilenceNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceNoticeStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceNoticeStatus) -> bool {
        match self.status {
            SilenceNoticeStatus::Draft => {
                return next == SilenceNoticeStatus::Pending || next == SilenceNoticeStatus::Closed;
            }
            SilenceNoticeStatus::Pending => {
                match next {
                    SilenceNoticeStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceNoticeStatus::Active => {
                return next == SilenceNoticeStatus::Closed;
            }
            SilenceNoticeStatus::Suspended => {
                return next == SilenceNoticeStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "silence notice", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.version as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceNotice {
        SilenceNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            active: self.active,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a silence notice in status draft.
pub fn new_silence_notice(code: &str, title: &str) -> SilenceNotice {
    SilenceNotice {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        active: false,
        version: 0,
        status: SilenceNoticeStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_silence_notice("code-986", "title-501");
        assert_eq!(item.status, SilenceNoticeStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_silence_notice("code-986", "title-501");
        if item.move_to(SilenceNoticeStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceNoticeStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_silence_notice("code-986", "title-501");
        item.status = SilenceNoticeStatus::Closed;
        assert!(
            item.move_to(SilenceNoticeStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
