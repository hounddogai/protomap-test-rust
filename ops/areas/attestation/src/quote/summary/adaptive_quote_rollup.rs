use super::*;

use acme_kit::text;

/// Where a adaptive quote rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveQuoteRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveQuoteRollupStatus {
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

impl std::fmt::Display for AdaptiveQuoteRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive quote rollup as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteRollup {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub tenant_id: String,
    pub weight: f64,
    pub amount: f64,
    pub label: String,
    pub issuer: String,
    pub burst: i64,
    pub external_ref: String,
    pub cluster: String,
    pub status: AdaptiveQuoteRollupStatus,
}

impl AdaptiveQuoteRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveQuoteRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveQuoteRollupStatus) -> bool {
        match self.status {
            AdaptiveQuoteRollupStatus::Draft => {
                return next == AdaptiveQuoteRollupStatus::Active || next == AdaptiveQuoteRollupStatus::Closed;
            }
            AdaptiveQuoteRollupStatus::Pending => {
                match next {
                    AdaptiveQuoteRollupStatus::Suspended | AdaptiveQuoteRollupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveQuoteRollupStatus::Active => {
                match next {
                    AdaptiveQuoteRollupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveQuoteRollupStatus::Suspended => {
                match next {
                    AdaptiveQuoteRollupStatus::Closed => {
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

    pub fn move_to(&mut self, next: AdaptiveQuoteRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive quote rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.weight + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveQuoteRollup {
        AdaptiveQuoteRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            tenant_id: self.tenant_id.clone(),
            weight: self.weight,
            amount: self.amount,
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            burst: self.burst,
            external_ref: self.external_ref.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a adaptive quote rollup in status draft.
pub fn new_adaptive_quote_rollup(code: &str, name: &str) -> AdaptiveQuoteRollup {
    AdaptiveQuoteRollup {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        tenant_id: "".to_string(),
        weight: 0.0,
        amount: 0.0,
        label: "".to_string(),
        issuer: "".to_string(),
        burst: 0,
        external_ref: "".to_string(),
        cluster: "".to_string(),
        status: AdaptiveQuoteRollupStatus::Draft,
    }
}
