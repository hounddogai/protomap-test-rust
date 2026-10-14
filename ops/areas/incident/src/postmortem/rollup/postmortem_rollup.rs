use super::*;

use acme_kit::text;

/// Where a postmortem rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PostmortemRollupStatus {
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

impl std::fmt::Display for PostmortemRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRollup {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub source: String,
    pub rate: f64,
    pub status: PostmortemRollupStatus,
}

impl PostmortemRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemRollupStatus) -> bool {
        match self.status {
            PostmortemRollupStatus::Draft => {
                match next {
                    PostmortemRollupStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemRollupStatus::Pending => {
                return next == PostmortemRollupStatus::Closed;
            }
            PostmortemRollupStatus::Active => {
                match next {
                    PostmortemRollupStatus::Suspended | PostmortemRollupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemRollupStatus::Suspended => {
                match next {
                    PostmortemRollupStatus::Closed => {
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

    pub fn move_to(&mut self, next: PostmortemRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "postmortem rollup", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.duration_minutes as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemRollup {
        PostmortemRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            source: self.source.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a postmortem rollup in status draft.
pub fn new_postmortem_rollup(code: &str, label: &str) -> PostmortemRollup {
    PostmortemRollup {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        source: "".to_string(),
        rate: 0.0,
        status: PostmortemRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_postmortem_rollup("code-172", "label-453");
        assert_eq!(item.status, PostmortemRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_postmortem_rollup("code-172", "label-453");
        if item.move_to(PostmortemRollupStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemRollupStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_postmortem_rollup("code-172", "label-453");
        item.status = PostmortemRollupStatus::Closed;
        assert!(
            item.move_to(PostmortemRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
