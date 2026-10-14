use super::*;

use acme_kit::text;

/// Where a quote bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QuoteBundleStatus {
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

impl std::fmt::Display for QuoteBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBundle {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub key_id: String,
    pub principal: String,
    pub active: bool,
    pub rotation_days: i64,
    pub duration_minutes: i64,
    pub expires_at: i64,
    pub status: QuoteBundleStatus,
}

impl QuoteBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteBundleStatus) -> bool {
        match self.status {
            QuoteBundleStatus::New => {
                match next {
                    QuoteBundleStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteBundleStatus::Queued => {
                return next == QuoteBundleStatus::Done;
            }
            QuoteBundleStatus::Running => {
                return next == QuoteBundleStatus::Done || next == QuoteBundleStatus::Failed;
            }
            QuoteBundleStatus::Done => {
                match next {
                    QuoteBundleStatus::Failed => {
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

    pub fn move_to(&mut self, next: QuoteBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote bundle {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.rotation_days as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteBundle {
        QuoteBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            key_id: self.key_id.clone(),
            principal: self.principal.clone(),
            active: self.active,
            rotation_days: self.rotation_days,
            duration_minutes: self.duration_minutes,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a quote bundle in status new.
pub fn new_quote_bundle(code: &str, title: &str) -> QuoteBundle {
    QuoteBundle {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        key_id: "".to_string(),
        principal: "".to_string(),
        active: false,
        rotation_days: 0,
        duration_minutes: 0,
        expires_at: 0,
        status: QuoteBundleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_quote_bundle("code-209", "title-491");
        assert_eq!(item.status, QuoteBundleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_quote_bundle("code-209", "title-491");
        if item.move_to(QuoteBundleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteBundleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_quote_bundle("code-209", "title-491");
        item.status = QuoteBundleStatus::Failed;
        assert!(item.move_to(QuoteBundleStatus::New).is_err(), "expected move to fail");
    }
}
