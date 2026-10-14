use super::*;

use acme_kit::text;

/// Where a redaction bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RedactionBundleStatus {
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

impl std::fmt::Display for RedactionBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction bundle as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBundle {
    pub code: String,
    pub title: String,
    pub category: String,
    pub created_at: i64,
    pub maximum: f64,
    pub expires_at: i64,
    pub status: RedactionBundleStatus,
}

impl RedactionBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionBundleStatus) -> bool {
        match self.status {
            RedactionBundleStatus::New => {
                match next {
                    RedactionBundleStatus::Queued | RedactionBundleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionBundleStatus::Queued => {
                return next == RedactionBundleStatus::Done;
            }
            RedactionBundleStatus::Running => {
                match next {
                    RedactionBundleStatus::Done | RedactionBundleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionBundleStatus::Done => {
                match next {
                    RedactionBundleStatus::Failed => {
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

    pub fn move_to(&mut self, next: RedactionBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.created_at as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionBundle {
        RedactionBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            created_at: self.created_at,
            maximum: self.maximum,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a redaction bundle in status new.
pub fn new_redaction_bundle(code: &str, title: &str) -> RedactionBundle {
    RedactionBundle {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        created_at: 0,
        maximum: 0.0,
        expires_at: 0,
        status: RedactionBundleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_redaction_bundle("code-350", "title-807");
        assert_eq!(item.status, RedactionBundleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_redaction_bundle("code-350", "title-807");
        if item.move_to(RedactionBundleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionBundleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_redaction_bundle("code-350", "title-807");
        item.status = RedactionBundleStatus::Failed;
        assert!(
            item.move_to(RedactionBundleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
