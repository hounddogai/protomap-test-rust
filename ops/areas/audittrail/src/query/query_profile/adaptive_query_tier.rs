use super::*;

use acme_kit::text;

/// Where a adaptive query tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveQueryTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveQueryTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveQueryTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive query tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryTier {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub locale: String,
    pub ends_at: i64,
    pub window_days: i64,
    pub updated_at: i64,
    pub summary: String,
    pub status: AdaptiveQueryTierStatus,
}

impl AdaptiveQueryTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveQueryTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveQueryTierStatus) -> bool {
        match self.status {
            AdaptiveQueryTierStatus::Proposed => {
                match next {
                    AdaptiveQueryTierStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveQueryTierStatus::Approved => {
                match next {
                    AdaptiveQueryTierStatus::Rejected | AdaptiveQueryTierStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveQueryTierStatus::Rejected => {
                match next {
                    AdaptiveQueryTierStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AdaptiveQueryTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive query tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive query tier", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_load(&self) -> f64 {
        ((self.window_days as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveQueryTier {
        AdaptiveQueryTier {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            locale: self.locale.clone(),
            ends_at: self.ends_at,
            window_days: self.window_days,
            updated_at: self.updated_at,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a adaptive query tier in status proposed.
pub fn new_adaptive_query_tier(code: &str, label: &str) -> AdaptiveQueryTier {
    AdaptiveQueryTier {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        locale: "".to_string(),
        ends_at: 0,
        window_days: 0,
        updated_at: 0,
        summary: "".to_string(),
        status: AdaptiveQueryTierStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_query_tier("code-310", "label-533");
        assert_eq!(item.status, AdaptiveQueryTierStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_adaptive_query_tier("code-310", "label-533");
        if item.move_to(AdaptiveQueryTierStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveQueryTierStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_query_tier("code-310", "label-533");
        item.status = AdaptiveQueryTierStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveQueryTierStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
