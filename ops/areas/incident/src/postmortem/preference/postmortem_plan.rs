use super::*;

use acme_kit::text;

/// Where a postmortem plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemPlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PostmortemPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PostmortemPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPlan {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub title: String,
    pub budget: f64,
    pub channel: String,
    pub category: String,
    pub created_at: i64,
    pub ratio: f64,
    pub ceiling: f64,
    pub cipher: String,
    pub status: PostmortemPlanStatus,
}

impl PostmortemPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemPlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemPlanStatus) -> bool {
        match self.status {
            PostmortemPlanStatus::Requested => {
                return next == PostmortemPlanStatus::Granted || next == PostmortemPlanStatus::Revoked;
            }
            PostmortemPlanStatus::Granted => {
                return next == PostmortemPlanStatus::Revoked;
            }
            PostmortemPlanStatus::Revoked => {
                match next {
                    PostmortemPlanStatus::Expired => {
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

    pub fn move_to(&mut self, next: PostmortemPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem plan {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.port as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemPlan {
        PostmortemPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            title: self.title.clone(),
            budget: self.budget,
            channel: self.channel.clone(),
            category: self.category.clone(),
            created_at: self.created_at,
            ratio: self.ratio,
            ceiling: self.ceiling,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a postmortem plan in status requested.
pub fn new_postmortem_plan(code: &str, name: &str) -> PostmortemPlan {
    PostmortemPlan {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        title: "".to_string(),
        budget: 0.0,
        channel: "".to_string(),
        category: "".to_string(),
        created_at: 0,
        ratio: 0.0,
        ceiling: 0.0,
        cipher: "".to_string(),
        status: PostmortemPlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_postmortem_plan("code-154", "name-136");
        assert_eq!(item.status, PostmortemPlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_postmortem_plan("code-154", "name-136");
        if item.move_to(PostmortemPlanStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemPlanStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_postmortem_plan("code-154", "name-136");
        item.status = PostmortemPlanStatus::Expired;
        assert!(
            item.move_to(PostmortemPlanStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
