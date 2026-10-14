use super::*;

use acme_kit::text;

/// Where a postmortem mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PostmortemMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PostmortemMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemMapping {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub phone: String,
    pub retries: i64,
    pub external_ref: String,
    pub visible: bool,
    pub status: PostmortemMappingStatus,
}

impl PostmortemMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemMappingStatus) -> bool {
        match self.status {
            PostmortemMappingStatus::Requested => {
                return next == PostmortemMappingStatus::Revoked;
            }
            PostmortemMappingStatus::Granted => {
                match next {
                    PostmortemMappingStatus::Revoked | PostmortemMappingStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemMappingStatus::Revoked => {
                match next {
                    PostmortemMappingStatus::Expired => {
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

    pub fn move_to(&mut self, next: PostmortemMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "postmortem mapping", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.priority as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemMapping {
        PostmortemMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            phone: self.phone.clone(),
            retries: self.retries,
            external_ref: self.external_ref.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a postmortem mapping in status requested.
pub fn new_postmortem_mapping(code: &str, title: &str) -> PostmortemMapping {
    PostmortemMapping {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        phone: "".to_string(),
        retries: 0,
        external_ref: "".to_string(),
        visible: false,
        status: PostmortemMappingStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_postmortem_mapping("code-178", "title-904");
        assert_eq!(item.status, PostmortemMappingStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_postmortem_mapping("code-178", "title-904");
        if item.move_to(PostmortemMappingStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemMappingStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_postmortem_mapping("code-178", "title-904");
        item.status = PostmortemMappingStatus::Expired;
        assert!(
            item.move_to(PostmortemMappingStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
