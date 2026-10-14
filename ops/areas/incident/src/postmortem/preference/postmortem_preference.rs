use super::*;

use acme_kit::text;

/// Where a postmortem preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PostmortemPreferenceStatus {
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

impl std::fmt::Display for PostmortemPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One postmortem preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPreference {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub currency: String,
    pub limit: i64,
    pub sequence: i64,
    pub audience: String,
    pub status: PostmortemPreferenceStatus,
}

impl PostmortemPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemPreferenceStatus) -> bool {
        match self.status {
            PostmortemPreferenceStatus::Draft => {
                return next == PostmortemPreferenceStatus::Active;
            }
            PostmortemPreferenceStatus::Pending => {
                return next == PostmortemPreferenceStatus::Active || next == PostmortemPreferenceStatus::Suspended;
            }
            PostmortemPreferenceStatus::Active => {
                match next {
                    PostmortemPreferenceStatus::Suspended | PostmortemPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemPreferenceStatus::Suspended => {
                match next {
                    PostmortemPreferenceStatus::Closed => {
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

    pub fn move_to(&mut self, next: PostmortemPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "postmortem preference", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.sequence as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemPreference {
        PostmortemPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            currency: self.currency.clone(),
            limit: self.limit,
            sequence: self.sequence,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a postmortem preference in status draft.
pub fn new_postmortem_preference(code: &str, title: &str) -> PostmortemPreference {
    PostmortemPreference {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        currency: "".to_string(),
        limit: 0,
        sequence: 0,
        audience: "".to_string(),
        status: PostmortemPreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_postmortem_preference("code-834", "title-296");
        assert_eq!(item.status, PostmortemPreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_postmortem_preference("code-834", "title-296");
        if item.move_to(PostmortemPreferenceStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemPreferenceStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_postmortem_preference("code-834", "title-296");
        item.status = PostmortemPreferenceStatus::Closed;
        assert!(
            item.move_to(PostmortemPreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
