use super::*;

use acme_kit::text;

/// Where a postmortem adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemAdjustmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PostmortemAdjustmentStatus {
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

impl std::fmt::Display for PostmortemAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAdjustment {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub label: String,
    pub notes: String,
    pub margin: f64,
    pub budget: f64,
    pub cipher: String,
    pub namespace: String,
    pub status: PostmortemAdjustmentStatus,
}

impl PostmortemAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemAdjustmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemAdjustmentStatus) -> bool {
        match self.status {
            PostmortemAdjustmentStatus::Draft => {
                match next {
                    PostmortemAdjustmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemAdjustmentStatus::Pending => {
                match next {
                    PostmortemAdjustmentStatus::Active | PostmortemAdjustmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemAdjustmentStatus::Active => {
                return next == PostmortemAdjustmentStatus::Suspended || next == PostmortemAdjustmentStatus::Closed;
            }
            PostmortemAdjustmentStatus::Suspended => {
                match next {
                    PostmortemAdjustmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: PostmortemAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem adjustment {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "postmortem adjustment", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.budget + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemAdjustment {
        PostmortemAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            label: self.label.clone(),
            notes: self.notes.clone(),
            margin: self.margin,
            budget: self.budget,
            cipher: self.cipher.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a postmortem adjustment in status draft.
pub fn new_postmortem_adjustment(code: &str, title: &str) -> PostmortemAdjustment {
    PostmortemAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        label: "".to_string(),
        notes: "".to_string(),
        margin: 0.0,
        budget: 0.0,
        cipher: "".to_string(),
        namespace: "".to_string(),
        status: PostmortemAdjustmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_postmortem_adjustment("code-59", "title-622");
        assert_eq!(item.status, PostmortemAdjustmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_postmortem_adjustment("code-59", "title-622");
        if item.move_to(PostmortemAdjustmentStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemAdjustmentStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_postmortem_adjustment("code-59", "title-622");
        item.status = PostmortemAdjustmentStatus::Closed;
        assert!(
            item.move_to(PostmortemAdjustmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
