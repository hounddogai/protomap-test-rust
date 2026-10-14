use super::*;

use acme_kit::text;

/// Where a pending sink review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSinkReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingSinkReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingSinkReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending sink review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSinkReview {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub attempts: i64,
    pub host: String,
    pub score: f64,
    pub contact_email: String,
    pub key_id: String,
    pub visible: bool,
    pub batch_size: i64,
    pub amount: f64,
    pub status: PendingSinkReviewStatus,
}

impl PendingSinkReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSinkReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSinkReviewStatus) -> bool {
        match self.status {
            PendingSinkReviewStatus::Created => {
                match next {
                    PendingSinkReviewStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSinkReviewStatus::Validated => {
                return next == PendingSinkReviewStatus::Published;
            }
            PendingSinkReviewStatus::Published => {
                match next {
                    PendingSinkReviewStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingSinkReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending sink review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.attempts as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingSinkReview {
        PendingSinkReview {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            attempts: self.attempts,
            host: self.host.clone(),
            score: self.score,
            contact_email: self.contact_email.clone(),
            key_id: self.key_id.clone(),
            visible: self.visible,
            batch_size: self.batch_size,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending sink review in status created.
pub fn new_pending_sink_review(code: &str, title: &str) -> PendingSinkReview {
    PendingSinkReview {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        attempts: 0,
        host: "".to_string(),
        score: 0.0,
        contact_email: "".to_string(),
        key_id: "".to_string(),
        visible: false,
        batch_size: 0,
        amount: 0.0,
        status: PendingSinkReviewStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_sink_review("code-28", "title-489");
        assert_eq!(item.status, PendingSinkReviewStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_pending_sink_review("code-28", "title-489");
        if item.move_to(PendingSinkReviewStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingSinkReviewStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_sink_review("code-28", "title-489");
        item.status = PendingSinkReviewStatus::Retired;
        assert!(
            item.move_to(PendingSinkReviewStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
