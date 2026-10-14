use super::*;

use acme_kit::text;

/// Where a scheduled sink review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSinkReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledSinkReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledSinkReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled sink review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkReview {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub pinned: bool,
    pub updated_at: i64,
    pub phone: String,
    pub description: String,
    pub rotation_days: i64,
    pub notes: String,
    pub attempts: i64,
    pub timezone: String,
    pub status: ScheduledSinkReviewStatus,
}

impl ScheduledSinkReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSinkReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSinkReviewStatus) -> bool {
        match self.status {
            ScheduledSinkReviewStatus::Created => {
                match next {
                    ScheduledSinkReviewStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSinkReviewStatus::Validated => {
                match next {
                    ScheduledSinkReviewStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSinkReviewStatus::Published => {
                match next {
                    ScheduledSinkReviewStatus::Retired => {
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

    pub fn move_to(&mut self, next: ScheduledSinkReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled sink review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.version as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSinkReview {
        ScheduledSinkReview {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            pinned: self.pinned,
            updated_at: self.updated_at,
            phone: self.phone.clone(),
            description: self.description.clone(),
            rotation_days: self.rotation_days,
            notes: self.notes.clone(),
            attempts: self.attempts,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled sink review in status created.
pub fn new_scheduled_sink_review(code: &str, title: &str) -> ScheduledSinkReview {
    ScheduledSinkReview {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        pinned: false,
        updated_at: 0,
        phone: "".to_string(),
        description: "".to_string(),
        rotation_days: 0,
        notes: "".to_string(),
        attempts: 0,
        timezone: "".to_string(),
        status: ScheduledSinkReviewStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_sink_review("code-600", "title-883");
        assert_eq!(item.status, ScheduledSinkReviewStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_scheduled_sink_review("code-600", "title-883");
        if item.move_to(ScheduledSinkReviewStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSinkReviewStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_sink_review("code-600", "title-883");
        item.status = ScheduledSinkReviewStatus::Retired;
        assert!(
            item.move_to(ScheduledSinkReviewStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
