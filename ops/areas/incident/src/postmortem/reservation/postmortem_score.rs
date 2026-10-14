use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a postmortem score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PostmortemScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PostmortemScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a postmortem score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemScore {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub rate: f64,
    pub attributes: BTreeMap<String, String>,
    pub tenant_id: String,
    pub owner_id: String,
    pub name: String,
    pub minimum: f64,
    pub amount: f64,
    pub status: PostmortemScoreStatus,
}

impl PostmortemScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemScoreStatus) -> bool {
        match self.status {
            PostmortemScoreStatus::New => {
                return next == PostmortemScoreStatus::Queued || next == PostmortemScoreStatus::Running;
            }
            PostmortemScoreStatus::Queued => {
                return next == PostmortemScoreStatus::Failed;
            }
            PostmortemScoreStatus::Running => {
                match next {
                    PostmortemScoreStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemScoreStatus::Done => {
                match next {
                    PostmortemScoreStatus::Failed => {
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

    pub fn move_to(&mut self, next: PostmortemScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rate - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemScore {
        PostmortemScore {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            rate: self.rate,
            attributes: self.attributes.clone(),
            tenant_id: self.tenant_id.clone(),
            owner_id: self.owner_id.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a postmortem score in status new.
pub fn new_postmortem_score(code: &str, title: &str) -> PostmortemScore {
    PostmortemScore {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        rate: 0.0,
        attributes: BTreeMap::new(),
        tenant_id: "".to_string(),
        owner_id: "".to_string(),
        name: "".to_string(),
        minimum: 0.0,
        amount: 0.0,
        status: PostmortemScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_postmortem_score("code-553", "title-84");
        assert_eq!(item.status, PostmortemScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_postmortem_score("code-553", "title-84");
        if item.move_to(PostmortemScoreStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemScoreStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_postmortem_score("code-553", "title-84");
        item.status = PostmortemScoreStatus::Failed;
        assert!(
            item.move_to(PostmortemScoreStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
