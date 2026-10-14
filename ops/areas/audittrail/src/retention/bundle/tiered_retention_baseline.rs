use super::*;

use acme_kit::text;

/// Where a tiered retention baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRetentionBaselineStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredRetentionBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredRetentionBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered retention baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionBaseline {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub score: f64,
    pub revision: i64,
    pub description: String,
    pub discount: f64,
    pub channel: String,
    pub quantity: i64,
    pub locale: String,
    pub external_ref: String,
    pub status: TieredRetentionBaselineStatus,
}

impl TieredRetentionBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRetentionBaselineStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRetentionBaselineStatus) -> bool {
        match self.status {
            TieredRetentionBaselineStatus::Proposed => {
                return next == TieredRetentionBaselineStatus::Rejected;
            }
            TieredRetentionBaselineStatus::Approved => {
                match next {
                    TieredRetentionBaselineStatus::Rejected | TieredRetentionBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRetentionBaselineStatus::Rejected => {
                match next {
                    TieredRetentionBaselineStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredRetentionBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered retention baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.score * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRetentionBaseline {
        TieredRetentionBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            score: self.score,
            revision: self.revision,
            description: self.description.clone(),
            discount: self.discount,
            channel: self.channel.clone(),
            quantity: self.quantity,
            locale: self.locale.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a tiered retention baseline in status proposed.
pub fn new_tiered_retention_baseline(code: &str, label: &str) -> TieredRetentionBaseline {
    TieredRetentionBaseline {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        score: 0.0,
        revision: 0,
        description: "".to_string(),
        discount: 0.0,
        channel: "".to_string(),
        quantity: 0,
        locale: "".to_string(),
        external_ref: "".to_string(),
        status: TieredRetentionBaselineStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_retention_baseline("code-913", "label-393");
        assert_eq!(item.status, TieredRetentionBaselineStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_tiered_retention_baseline("code-913", "label-393");
        if item.move_to(TieredRetentionBaselineStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRetentionBaselineStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_retention_baseline("code-913", "label-393");
        item.status = TieredRetentionBaselineStatus::Withdrawn;
        assert!(
            item.move_to(TieredRetentionBaselineStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
