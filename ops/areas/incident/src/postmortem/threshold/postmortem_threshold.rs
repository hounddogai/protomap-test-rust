use super::*;

use acme_kit::text;

/// Where a postmortem threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PostmortemThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PostmortemThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem threshold as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemThreshold {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub created_at: i64,
    pub quantity: i64,
    pub weight: f64,
    pub checksum: String,
    pub visible: bool,
    pub status: PostmortemThresholdStatus,
}

impl PostmortemThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemThresholdStatus) -> bool {
        match self.status {
            PostmortemThresholdStatus::Draft => {
                return next == PostmortemThresholdStatus::Suspended;
            }
            PostmortemThresholdStatus::Pending => {
                match next {
                    PostmortemThresholdStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemThresholdStatus::Active => {
                return next == PostmortemThresholdStatus::Closed;
            }
            PostmortemThresholdStatus::Suspended => {
                return next == PostmortemThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem threshold {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.weight + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemThreshold {
        PostmortemThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            created_at: self.created_at,
            quantity: self.quantity,
            weight: self.weight,
            checksum: self.checksum.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a postmortem threshold in status draft.
pub fn new_postmortem_threshold(code: &str, title: &str) -> PostmortemThreshold {
    PostmortemThreshold {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        created_at: 0,
        quantity: 0,
        weight: 0.0,
        checksum: "".to_string(),
        visible: false,
        status: PostmortemThresholdStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_postmortem_threshold("code-974", "title-324");
        assert_eq!(item.status, PostmortemThresholdStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_postmortem_threshold("code-974", "title-324");
        if item.move_to(PostmortemThresholdStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemThresholdStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_postmortem_threshold("code-974", "title-324");
        item.status = PostmortemThresholdStatus::Closed;
        assert!(
            item.move_to(PostmortemThresholdStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
