use super::*;

use acme_kit::text;

/// Where a postmortem policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemPolicyStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PostmortemPolicyStatus {
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

impl std::fmt::Display for PostmortemPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPolicy {
    pub code: String,
    pub title: String,
    pub source: String,
    pub fingerprint: String,
    pub owner_id: String,
    pub priority: i64,
    pub attempts: i64,
    pub host: String,
    pub maximum: f64,
    pub visible: bool,
    pub status: PostmortemPolicyStatus,
}

impl PostmortemPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemPolicyStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemPolicyStatus) -> bool {
        match self.status {
            PostmortemPolicyStatus::Draft => {
                return next == PostmortemPolicyStatus::Suspended;
            }
            PostmortemPolicyStatus::Pending => {
                match next {
                    PostmortemPolicyStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemPolicyStatus::Active => {
                return next == PostmortemPolicyStatus::Suspended || next == PostmortemPolicyStatus::Closed;
            }
            PostmortemPolicyStatus::Suspended => {
                match next {
                    PostmortemPolicyStatus::Closed => {
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

    pub fn move_to(&mut self, next: PostmortemPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.maximum + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemPolicy {
        PostmortemPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            fingerprint: self.fingerprint.clone(),
            owner_id: self.owner_id.clone(),
            priority: self.priority,
            attempts: self.attempts,
            host: self.host.clone(),
            maximum: self.maximum,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a postmortem policy in status draft.
pub fn new_postmortem_policy(code: &str, title: &str) -> PostmortemPolicy {
    PostmortemPolicy {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        fingerprint: "".to_string(),
        owner_id: "".to_string(),
        priority: 0,
        attempts: 0,
        host: "".to_string(),
        maximum: 0.0,
        visible: false,
        status: PostmortemPolicyStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_postmortem_policy("code-870", "title-695");
        assert_eq!(item.status, PostmortemPolicyStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_postmortem_policy("code-870", "title-695");
        if item.move_to(PostmortemPolicyStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemPolicyStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_postmortem_policy("code-870", "title-695");
        item.status = PostmortemPolicyStatus::Closed;
        assert!(
            item.move_to(PostmortemPolicyStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
