use super::*;

use acme_kit::text;

/// Where a regional quote adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalQuoteAdjustmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalQuoteAdjustmentStatus {
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

impl std::fmt::Display for RegionalQuoteAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional quote adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteAdjustment {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub ratio: f64,
    pub approved: bool,
    pub key_id: String,
    pub retries: i64,
    pub issuer: String,
    pub source: String,
    pub reviewer: String,
    pub status: RegionalQuoteAdjustmentStatus,
}

impl RegionalQuoteAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalQuoteAdjustmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalQuoteAdjustmentStatus) -> bool {
        match self.status {
            RegionalQuoteAdjustmentStatus::Draft => {
                return next == RegionalQuoteAdjustmentStatus::Active || next == RegionalQuoteAdjustmentStatus::Closed;
            }
            RegionalQuoteAdjustmentStatus::Pending => {
                return next == RegionalQuoteAdjustmentStatus::Suspended;
            }
            RegionalQuoteAdjustmentStatus::Active => {
                match next {
                    RegionalQuoteAdjustmentStatus::Suspended | RegionalQuoteAdjustmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalQuoteAdjustmentStatus::Suspended => {
                return next == RegionalQuoteAdjustmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalQuoteAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional quote adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.retries as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalQuoteAdjustment {
        RegionalQuoteAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            ratio: self.ratio,
            approved: self.approved,
            key_id: self.key_id.clone(),
            retries: self.retries,
            issuer: self.issuer.clone(),
            source: self.source.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a regional quote adjustment in status draft.
pub fn new_regional_quote_adjustment(code: &str, label: &str) -> RegionalQuoteAdjustment {
    RegionalQuoteAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        ratio: 0.0,
        approved: false,
        key_id: "".to_string(),
        retries: 0,
        issuer: "".to_string(),
        source: "".to_string(),
        reviewer: "".to_string(),
        status: RegionalQuoteAdjustmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_quote_adjustment("code-755", "label-249");
        assert_eq!(item.status, RegionalQuoteAdjustmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_regional_quote_adjustment("code-755", "label-249");
        if item.move_to(RegionalQuoteAdjustmentStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalQuoteAdjustmentStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_quote_adjustment("code-755", "label-249");
        item.status = RegionalQuoteAdjustmentStatus::Closed;
        assert!(
            item.move_to(RegionalQuoteAdjustmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
