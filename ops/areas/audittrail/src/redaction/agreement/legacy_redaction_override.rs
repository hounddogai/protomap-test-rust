use super::*;

use acme_kit::text;

/// Where a legacy redaction override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRedactionOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyRedactionOverrideStatus {
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

impl std::fmt::Display for LegacyRedactionOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy redaction override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionOverride {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub starts_at: i64,
    pub checksum: String,
    pub budget: f64,
    pub summary: String,
    pub cost: f64,
    pub maximum: f64,
    pub quota_units: i64,
    pub status: LegacyRedactionOverrideStatus,
}

impl LegacyRedactionOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRedactionOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRedactionOverrideStatus) -> bool {
        match self.status {
            LegacyRedactionOverrideStatus::Draft => {
                match next {
                    LegacyRedactionOverrideStatus::Active | LegacyRedactionOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRedactionOverrideStatus::Pending => {
                match next {
                    LegacyRedactionOverrideStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRedactionOverrideStatus::Active => {
                return next == LegacyRedactionOverrideStatus::Suspended;
            }
            LegacyRedactionOverrideStatus::Suspended => {
                match next {
                    LegacyRedactionOverrideStatus::Closed => {
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

    pub fn move_to(&mut self, next: LegacyRedactionOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy redaction override {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.maximum + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRedactionOverride {
        LegacyRedactionOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            starts_at: self.starts_at,
            checksum: self.checksum.clone(),
            budget: self.budget,
            summary: self.summary.clone(),
            cost: self.cost,
            maximum: self.maximum,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy redaction override in status draft.
pub fn new_legacy_redaction_override(code: &str, title: &str) -> LegacyRedactionOverride {
    LegacyRedactionOverride {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        starts_at: 0,
        checksum: "".to_string(),
        budget: 0.0,
        summary: "".to_string(),
        cost: 0.0,
        maximum: 0.0,
        quota_units: 0,
        status: LegacyRedactionOverrideStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_redaction_override("code-141", "title-320");
        assert_eq!(item.status, LegacyRedactionOverrideStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_legacy_redaction_override("code-141", "title-320");
        if item.move_to(LegacyRedactionOverrideStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRedactionOverrideStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_redaction_override("code-141", "title-320");
        item.status = LegacyRedactionOverrideStatus::Closed;
        assert!(
            item.move_to(LegacyRedactionOverrideStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
