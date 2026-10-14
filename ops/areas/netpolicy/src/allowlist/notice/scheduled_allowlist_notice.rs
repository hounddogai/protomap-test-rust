use super::*;

use acme_kit::text;

/// Where a scheduled allowlist notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledAllowlistNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledAllowlistNoticeStatus {
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

impl std::fmt::Display for ScheduledAllowlistNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled allowlist notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistNotice {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub fingerprint: String,
    pub limit: i64,
    pub sequence: i64,
    pub cluster: String,
    pub key_id: String,
    pub reviewer: String,
    pub status: ScheduledAllowlistNoticeStatus,
}

impl ScheduledAllowlistNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledAllowlistNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledAllowlistNoticeStatus) -> bool {
        match self.status {
            ScheduledAllowlistNoticeStatus::New => {
                return next == ScheduledAllowlistNoticeStatus::Done;
            }
            ScheduledAllowlistNoticeStatus::Queued => {
                return next == ScheduledAllowlistNoticeStatus::Failed;
            }
            ScheduledAllowlistNoticeStatus::Running => {
                match next {
                    ScheduledAllowlistNoticeStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledAllowlistNoticeStatus::Done => {
                return next == ScheduledAllowlistNoticeStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledAllowlistNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled allowlist notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled allowlist notice", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.limit as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledAllowlistNotice {
        ScheduledAllowlistNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            fingerprint: self.fingerprint.clone(),
            limit: self.limit,
            sequence: self.sequence,
            cluster: self.cluster.clone(),
            key_id: self.key_id.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a scheduled allowlist notice in status new.
pub fn new_scheduled_allowlist_notice(code: &str, title: &str) -> ScheduledAllowlistNotice {
    ScheduledAllowlistNotice {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        fingerprint: "".to_string(),
        limit: 0,
        sequence: 0,
        cluster: "".to_string(),
        key_id: "".to_string(),
        reviewer: "".to_string(),
        status: ScheduledAllowlistNoticeStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_allowlist_notice("code-675", "title-265");
        assert_eq!(item.status, ScheduledAllowlistNoticeStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_scheduled_allowlist_notice("code-675", "title-265");
        if item.move_to(ScheduledAllowlistNoticeStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledAllowlistNoticeStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_allowlist_notice("code-675", "title-265");
        item.status = ScheduledAllowlistNoticeStatus::Failed;
        assert!(
            item.move_to(ScheduledAllowlistNoticeStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
