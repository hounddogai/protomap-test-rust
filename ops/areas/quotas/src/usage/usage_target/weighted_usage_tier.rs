use super::*;

use acme_kit::text;

/// Where a weighted usage tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedUsageTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedUsageTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for WeightedUsageTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted usage tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTier {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub name: String,
    pub rate: f64,
    pub score: f64,
    pub starts_at: i64,
    pub checksum: String,
    pub status: WeightedUsageTierStatus,
}

impl WeightedUsageTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedUsageTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedUsageTierStatus) -> bool {
        match self.status {
            WeightedUsageTierStatus::Open => {
                return next == WeightedUsageTierStatus::InReview || next == WeightedUsageTierStatus::Archived;
            }
            WeightedUsageTierStatus::InReview => {
                return next == WeightedUsageTierStatus::Archived;
            }
            WeightedUsageTierStatus::Resolved => {
                return next == WeightedUsageTierStatus::Archived;
            }
            WeightedUsageTierStatus::Reopened => {
                match next {
                    WeightedUsageTierStatus::Archived => {
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

    pub fn move_to(&mut self, next: WeightedUsageTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted usage tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.rate + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedUsageTier {
        WeightedUsageTier {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            name: self.name.clone(),
            rate: self.rate,
            score: self.score,
            starts_at: self.starts_at,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted usage tier in status open.
pub fn new_weighted_usage_tier(code: &str, title: &str) -> WeightedUsageTier {
    WeightedUsageTier {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        name: "".to_string(),
        rate: 0.0,
        score: 0.0,
        starts_at: 0,
        checksum: "".to_string(),
        status: WeightedUsageTierStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_usage_tier("code-332", "title-361");
        assert_eq!(item.status, WeightedUsageTierStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_weighted_usage_tier("code-332", "title-361");
        if item.move_to(WeightedUsageTierStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedUsageTierStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_usage_tier("code-332", "title-361");
        item.status = WeightedUsageTierStatus::Archived;
        assert!(
            item.move_to(WeightedUsageTierStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
