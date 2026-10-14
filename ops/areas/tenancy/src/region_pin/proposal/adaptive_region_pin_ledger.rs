use super::*;

use acme_kit::text;

/// Where a adaptive region pin ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRegionPinLedgerStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveRegionPinLedgerStatus {
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

impl std::fmt::Display for AdaptiveRegionPinLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive region pin ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinLedger {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub cipher: String,
    pub segment: String,
    pub tags: Vec<String>,
    pub version: i64,
    pub tenant_id: String,
    pub status: AdaptiveRegionPinLedgerStatus,
}

impl AdaptiveRegionPinLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRegionPinLedgerStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRegionPinLedgerStatus) -> bool {
        match self.status {
            AdaptiveRegionPinLedgerStatus::Draft => {
                match next {
                    AdaptiveRegionPinLedgerStatus::Pending | AdaptiveRegionPinLedgerStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRegionPinLedgerStatus::Pending => {
                match next {
                    AdaptiveRegionPinLedgerStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRegionPinLedgerStatus::Active => {
                match next {
                    AdaptiveRegionPinLedgerStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRegionPinLedgerStatus::Suspended => {
                return next == AdaptiveRegionPinLedgerStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRegionPinLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive region pin ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive region pin ledger", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.version as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRegionPinLedger {
        AdaptiveRegionPinLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            cipher: self.cipher.clone(),
            segment: self.segment.clone(),
            tags: self.tags.clone(),
            version: self.version,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a adaptive region pin ledger in status draft.
pub fn new_adaptive_region_pin_ledger(code: &str, title: &str) -> AdaptiveRegionPinLedger {
    AdaptiveRegionPinLedger {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        cipher: "".to_string(),
        segment: "".to_string(),
        tags: Vec::new(),
        version: 0,
        tenant_id: "".to_string(),
        status: AdaptiveRegionPinLedgerStatus::Draft,
    }
}
