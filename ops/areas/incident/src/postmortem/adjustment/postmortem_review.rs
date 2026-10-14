use super::*;

use acme_kit::text;

/// Where a postmortem review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PostmortemReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for PostmortemReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReview {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub description: String,
    pub phone: String,
    pub updated_at: i64,
    pub name: String,
    pub key_id: String,
    pub minimum: f64,
    pub audience: String,
    pub created_at: i64,
    pub status: PostmortemReviewStatus,
}

impl PostmortemReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemReviewStatus) -> bool {
        match self.status {
            PostmortemReviewStatus::Idle => {
                return next == PostmortemReviewStatus::Draining;
            }
            PostmortemReviewStatus::Warming => {
                match next {
                    PostmortemReviewStatus::Draining | PostmortemReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemReviewStatus::Ready => {
                return next == PostmortemReviewStatus::Stopped;
            }
            PostmortemReviewStatus::Draining => {
                match next {
                    PostmortemReviewStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PostmortemReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.created_at as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemReview {
        PostmortemReview {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            description: self.description.clone(),
            phone: self.phone.clone(),
            updated_at: self.updated_at,
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            minimum: self.minimum,
            audience: self.audience.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a postmortem review in status idle.
pub fn new_postmortem_review(code: &str, title: &str) -> PostmortemReview {
    PostmortemReview {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        description: "".to_string(),
        phone: "".to_string(),
        updated_at: 0,
        name: "".to_string(),
        key_id: "".to_string(),
        minimum: 0.0,
        audience: "".to_string(),
        created_at: 0,
        status: PostmortemReviewStatus::Idle,
    }
}
