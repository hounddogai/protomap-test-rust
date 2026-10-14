use super::*;

use acme_kit::text;

/// Where a retention preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionPreferenceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RetentionPreferenceStatus {
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

impl std::fmt::Display for RetentionPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One retention preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPreference {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub currency: String,
    pub audience: String,
    pub score: f64,
    pub active: bool,
    pub ratio: f64,
    pub channel: String,
    pub status: RetentionPreferenceStatus,
}

impl RetentionPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionPreferenceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionPreferenceStatus) -> bool {
        match self.status {
            RetentionPreferenceStatus::New => {
                match next {
                    RetentionPreferenceStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionPreferenceStatus::Queued => {
                return next == RetentionPreferenceStatus::Done || next == RetentionPreferenceStatus::Failed;
            }
            RetentionPreferenceStatus::Running => {
                return next == RetentionPreferenceStatus::Failed;
            }
            RetentionPreferenceStatus::Done => {
                match next {
                    RetentionPreferenceStatus::Failed => {
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

    pub fn move_to(&mut self, next: RetentionPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "retention preference", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.ratio * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionPreference {
        RetentionPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            currency: self.currency.clone(),
            audience: self.audience.clone(),
            score: self.score,
            active: self.active,
            ratio: self.ratio,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a retention preference in status new.
pub fn new_retention_preference(code: &str, title: &str) -> RetentionPreference {
    RetentionPreference {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        currency: "".to_string(),
        audience: "".to_string(),
        score: 0.0,
        active: false,
        ratio: 0.0,
        channel: "".to_string(),
        status: RetentionPreferenceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_retention_preference("code-172", "title-953");
        assert_eq!(item.status, RetentionPreferenceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_retention_preference("code-172", "title-953");
        if item.move_to(RetentionPreferenceStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionPreferenceStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_retention_preference("code-172", "title-953");
        item.status = RetentionPreferenceStatus::Failed;
        assert!(
            item.move_to(RetentionPreferenceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
