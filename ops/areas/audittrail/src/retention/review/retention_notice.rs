use super::*;

use acme_kit::text;

/// Where a retention notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RetentionNoticeStatus {
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

impl std::fmt::Display for RetentionNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionNotice {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub duration_minutes: i64,
    pub locale: String,
    pub starts_at: i64,
    pub version: i64,
    pub margin: f64,
    pub budget: f64,
    pub amount: f64,
    pub contact_email: String,
    pub status: RetentionNoticeStatus,
}

impl RetentionNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionNoticeStatus) -> bool {
        match self.status {
            RetentionNoticeStatus::New => {
                match next {
                    RetentionNoticeStatus::Running | RetentionNoticeStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionNoticeStatus::Queued => {
                match next {
                    RetentionNoticeStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionNoticeStatus::Running => {
                return next == RetentionNoticeStatus::Done || next == RetentionNoticeStatus::Failed;
            }
            RetentionNoticeStatus::Done => {
                match next {
                    RetentionNoticeStatus::Failed => {
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

    pub fn move_to(&mut self, next: RetentionNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.version as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionNotice {
        RetentionNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            duration_minutes: self.duration_minutes,
            locale: self.locale.clone(),
            starts_at: self.starts_at,
            version: self.version,
            margin: self.margin,
            budget: self.budget,
            amount: self.amount,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a retention notice in status new.
pub fn new_retention_notice(code: &str, title: &str) -> RetentionNotice {
    RetentionNotice {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        duration_minutes: 0,
        locale: "".to_string(),
        starts_at: 0,
        version: 0,
        margin: 0.0,
        budget: 0.0,
        amount: 0.0,
        contact_email: "".to_string(),
        status: RetentionNoticeStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_retention_notice("code-846", "title-885");
        assert_eq!(item.status, RetentionNoticeStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_retention_notice("code-846", "title-885");
        if item.move_to(RetentionNoticeStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionNoticeStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_retention_notice("code-846", "title-885");
        item.status = RetentionNoticeStatus::Failed;
        assert!(
            item.move_to(RetentionNoticeStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
