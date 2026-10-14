use super::*;

use acme_kit::text;

/// Where a postmortem tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PostmortemTierStatus {
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

impl std::fmt::Display for PostmortemTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem tier as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemTier {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub created_at: i64,
    pub country: String,
    pub audience: String,
    pub ends_at: i64,
    pub status: PostmortemTierStatus,
}

impl PostmortemTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemTierStatus) -> bool {
        match self.status {
            PostmortemTierStatus::Draft => {
                match next {
                    PostmortemTierStatus::Active | PostmortemTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemTierStatus::Pending => {
                return next == PostmortemTierStatus::Active || next == PostmortemTierStatus::Suspended;
            }
            PostmortemTierStatus::Active => {
                return next == PostmortemTierStatus::Suspended;
            }
            PostmortemTierStatus::Suspended => {
                return next == PostmortemTierStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem tier {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.created_at as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemTier {
        PostmortemTier {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            created_at: self.created_at,
            country: self.country.clone(),
            audience: self.audience.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a postmortem tier in status draft.
pub fn new_postmortem_tier(code: &str, label: &str) -> PostmortemTier {
    PostmortemTier {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        created_at: 0,
        country: "".to_string(),
        audience: "".to_string(),
        ends_at: 0,
        status: PostmortemTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_postmortem_tier("code-231", "label-78");
        assert_eq!(item.status, PostmortemTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_postmortem_tier("code-231", "label-78");
        if item.move_to(PostmortemTierStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemTierStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_postmortem_tier("code-231", "label-78");
        item.status = PostmortemTierStatus::Closed;
        assert!(
            item.move_to(PostmortemTierStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
