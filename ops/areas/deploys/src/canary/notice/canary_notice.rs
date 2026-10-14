use super::*;

use acme_kit::text;

/// Where a canary notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CanaryNoticeStatus {
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

impl std::fmt::Display for CanaryNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryNotice {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub rate: f64,
    pub expires_at: i64,
    pub ends_at: i64,
    pub segment: String,
    pub region: String,
    pub capacity: i64,
    pub key_id: String,
    pub window_days: i64,
    pub status: CanaryNoticeStatus,
}

impl CanaryNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryNoticeStatus) -> bool {
        match self.status {
            CanaryNoticeStatus::New => {
                return next == CanaryNoticeStatus::Queued || next == CanaryNoticeStatus::Failed;
            }
            CanaryNoticeStatus::Queued => {
                return next == CanaryNoticeStatus::Running || next == CanaryNoticeStatus::Failed;
            }
            CanaryNoticeStatus::Running => {
                match next {
                    CanaryNoticeStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryNoticeStatus::Done => {
                return next == CanaryNoticeStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary notice {} cannot move from {} to {}",
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
        format!("{} {}", "canary notice", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.expires_at as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryNotice {
        CanaryNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            rate: self.rate,
            expires_at: self.expires_at,
            ends_at: self.ends_at,
            segment: self.segment.clone(),
            region: self.region.clone(),
            capacity: self.capacity,
            key_id: self.key_id.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a canary notice in status new.
pub fn new_canary_notice(code: &str, label: &str) -> CanaryNotice {
    CanaryNotice {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        rate: 0.0,
        expires_at: 0,
        ends_at: 0,
        segment: "".to_string(),
        region: "".to_string(),
        capacity: 0,
        key_id: "".to_string(),
        window_days: 0,
        status: CanaryNoticeStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_canary_notice("code-420", "label-515");
        assert_eq!(item.status, CanaryNoticeStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_canary_notice("code-420", "label-515");
        if item.move_to(CanaryNoticeStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryNoticeStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_canary_notice("code-420", "label-515");
        item.status = CanaryNoticeStatus::Failed;
        assert!(item.move_to(CanaryNoticeStatus::New).is_err(), "expected move to fail");
    }
}
