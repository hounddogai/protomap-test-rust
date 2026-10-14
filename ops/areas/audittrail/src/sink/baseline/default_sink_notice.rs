use super::*;

use acme_kit::text;

/// Where a default sink notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultSinkNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultSinkNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DefaultSinkNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default sink notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSinkNotice {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub attempts: i64,
    pub version: i64,
    pub limit: i64,
    pub phone: String,
    pub title: String,
    pub window_days: i64,
    pub issuer: String,
    pub ttl_seconds: i64,
    pub status: DefaultSinkNoticeStatus,
}

impl DefaultSinkNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultSinkNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultSinkNoticeStatus) -> bool {
        match self.status {
            DefaultSinkNoticeStatus::Planned => {
                return next == DefaultSinkNoticeStatus::Scheduled || next == DefaultSinkNoticeStatus::Cancelled;
            }
            DefaultSinkNoticeStatus::Scheduled => {
                return next == DefaultSinkNoticeStatus::InProgress || next == DefaultSinkNoticeStatus::Completed;
            }
            DefaultSinkNoticeStatus::InProgress => {
                match next {
                    DefaultSinkNoticeStatus::Completed | DefaultSinkNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSinkNoticeStatus::Completed => {
                return next == DefaultSinkNoticeStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultSinkNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default sink notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultSinkNotice {
        DefaultSinkNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            attempts: self.attempts,
            version: self.version,
            limit: self.limit,
            phone: self.phone.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            issuer: self.issuer.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default sink notice in status planned.
pub fn new_default_sink_notice(code: &str, name: &str) -> DefaultSinkNotice {
    DefaultSinkNotice {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        attempts: 0,
        version: 0,
        limit: 0,
        phone: "".to_string(),
        title: "".to_string(),
        window_days: 0,
        issuer: "".to_string(),
        ttl_seconds: 0,
        status: DefaultSinkNoticeStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_sink_notice("code-698", "name-228");
        assert_eq!(item.status, DefaultSinkNoticeStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_default_sink_notice("code-698", "name-228");
        if item.move_to(DefaultSinkNoticeStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultSinkNoticeStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_sink_notice("code-698", "name-228");
        item.status = DefaultSinkNoticeStatus::Cancelled;
        assert!(
            item.move_to(DefaultSinkNoticeStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
