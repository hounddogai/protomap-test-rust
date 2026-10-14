use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a tiered dashboard digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredDashboardDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredDashboardDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredDashboardDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered dashboard digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardDigest {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub maximum: f64,
    pub score: f64,
    pub grace_days: i64,
    pub tags: Vec<String>,
    pub checksum: String,
    pub attributes: BTreeMap<String, String>,
    pub label: String,
    pub status: TieredDashboardDigestStatus,
}

impl TieredDashboardDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredDashboardDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredDashboardDigestStatus) -> bool {
        match self.status {
            TieredDashboardDigestStatus::Proposed => {
                return next == TieredDashboardDigestStatus::Rejected || next == TieredDashboardDigestStatus::Withdrawn;
            }
            TieredDashboardDigestStatus::Approved => {
                return next == TieredDashboardDigestStatus::Rejected || next == TieredDashboardDigestStatus::Withdrawn;
            }
            TieredDashboardDigestStatus::Rejected => {
                match next {
                    TieredDashboardDigestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredDashboardDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered dashboard digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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

    pub fn net_load(&self) -> f64 {
        (self.score * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredDashboardDigest {
        TieredDashboardDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            maximum: self.maximum,
            score: self.score,
            grace_days: self.grace_days,
            tags: self.tags.clone(),
            checksum: self.checksum.clone(),
            attributes: self.attributes.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a tiered dashboard digest in status proposed.
pub fn new_tiered_dashboard_digest(code: &str, title: &str) -> TieredDashboardDigest {
    TieredDashboardDigest {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        maximum: 0.0,
        score: 0.0,
        grace_days: 0,
        tags: Vec::new(),
        checksum: "".to_string(),
        attributes: BTreeMap::new(),
        label: "".to_string(),
        status: TieredDashboardDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_dashboard_digest("code-994", "title-58");
        assert_eq!(item.status, TieredDashboardDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_tiered_dashboard_digest("code-994", "title-58");
        if item.move_to(TieredDashboardDigestStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredDashboardDigestStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_dashboard_digest("code-994", "title-58");
        item.status = TieredDashboardDigestStatus::Withdrawn;
        assert!(
            item.move_to(TieredDashboardDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
