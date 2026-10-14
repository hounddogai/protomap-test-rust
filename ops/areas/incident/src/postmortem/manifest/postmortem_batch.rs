use super::*;

use acme_kit::text;

/// Where a postmortem batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PostmortemBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PostmortemBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a postmortem batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBatch {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub ends_at: i64,
    pub ratio: f64,
    pub reviewer: String,
    pub quantity: i64,
    pub status: PostmortemBatchStatus,
}

impl PostmortemBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemBatchStatus) -> bool {
        match self.status {
            PostmortemBatchStatus::Requested => {
                match next {
                    PostmortemBatchStatus::Revoked | PostmortemBatchStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemBatchStatus::Granted => {
                return next == PostmortemBatchStatus::Revoked;
            }
            PostmortemBatchStatus::Revoked => {
                match next {
                    PostmortemBatchStatus::Expired => {
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

    pub fn move_to(&mut self, next: PostmortemBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "postmortem batch", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.budget + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemBatch {
        PostmortemBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            ends_at: self.ends_at,
            ratio: self.ratio,
            reviewer: self.reviewer.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a postmortem batch in status requested.
pub fn new_postmortem_batch(code: &str, title: &str) -> PostmortemBatch {
    PostmortemBatch {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        ends_at: 0,
        ratio: 0.0,
        reviewer: "".to_string(),
        quantity: 0,
        status: PostmortemBatchStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_postmortem_batch("code-321", "title-825");
        assert_eq!(item.status, PostmortemBatchStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_postmortem_batch("code-321", "title-825");
        if item.move_to(PostmortemBatchStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemBatchStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_postmortem_batch("code-321", "title-825");
        item.status = PostmortemBatchStatus::Expired;
        assert!(
            item.move_to(PostmortemBatchStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
