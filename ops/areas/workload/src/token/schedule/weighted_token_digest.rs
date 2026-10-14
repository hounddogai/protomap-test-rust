use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted token digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTokenDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedTokenDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedTokenDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted token digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenDigest {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub amount: f64,
    pub weight: f64,
    pub attributes: BTreeMap<String, String>,
    pub archived: bool,
    pub ceiling: f64,
    pub status: WeightedTokenDigestStatus,
}

impl WeightedTokenDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTokenDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTokenDigestStatus) -> bool {
        match self.status {
            WeightedTokenDigestStatus::Proposed => {
                return next == WeightedTokenDigestStatus::Rejected || next == WeightedTokenDigestStatus::Withdrawn;
            }
            WeightedTokenDigestStatus::Approved => {
                match next {
                    WeightedTokenDigestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTokenDigestStatus::Rejected => {
                return next == WeightedTokenDigestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedTokenDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted token digest {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.amount + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTokenDigest {
        WeightedTokenDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            created_at: self.created_at,
            amount: self.amount,
            weight: self.weight,
            attributes: self.attributes.clone(),
            archived: self.archived,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted token digest in status proposed.
pub fn new_weighted_token_digest(code: &str, title: &str) -> WeightedTokenDigest {
    WeightedTokenDigest {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        created_at: 0,
        amount: 0.0,
        weight: 0.0,
        attributes: BTreeMap::new(),
        archived: false,
        ceiling: 0.0,
        status: WeightedTokenDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_weighted_token_digest("code-315", "title-696");
        assert_eq!(item.status, WeightedTokenDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_weighted_token_digest("code-315", "title-696");
        if item.move_to(WeightedTokenDigestStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTokenDigestStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_weighted_token_digest("code-315", "title-696");
        item.status = WeightedTokenDigestStatus::Withdrawn;
        assert!(
            item.move_to(WeightedTokenDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
