use super::*;

use acme_kit::text;

/// Where a legacy allowlist rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyAllowlistRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyAllowlistRollupStatus {
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

impl std::fmt::Display for LegacyAllowlistRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy allowlist rollup as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistRollup {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub version: i64,
    pub limit: i64,
    pub host: String,
    pub region: String,
    pub updated_at: i64,
    pub locale: String,
    pub checksum: String,
    pub status: LegacyAllowlistRollupStatus,
}

impl LegacyAllowlistRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyAllowlistRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyAllowlistRollupStatus) -> bool {
        match self.status {
            LegacyAllowlistRollupStatus::Draft => {
                match next {
                    LegacyAllowlistRollupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistRollupStatus::Pending => {
                return next == LegacyAllowlistRollupStatus::Active || next == LegacyAllowlistRollupStatus::Suspended;
            }
            LegacyAllowlistRollupStatus::Active => {
                return next == LegacyAllowlistRollupStatus::Closed;
            }
            LegacyAllowlistRollupStatus::Suspended => {
                return next == LegacyAllowlistRollupStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyAllowlistRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy allowlist rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.discount + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyAllowlistRollup {
        LegacyAllowlistRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            version: self.version,
            limit: self.limit,
            host: self.host.clone(),
            region: self.region.clone(),
            updated_at: self.updated_at,
            locale: self.locale.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a legacy allowlist rollup in status draft.
pub fn new_legacy_allowlist_rollup(code: &str, label: &str) -> LegacyAllowlistRollup {
    LegacyAllowlistRollup {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        version: 0,
        limit: 0,
        host: "".to_string(),
        region: "".to_string(),
        updated_at: 0,
        locale: "".to_string(),
        checksum: "".to_string(),
        status: LegacyAllowlistRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_allowlist_rollup("code-683", "label-439");
        assert_eq!(item.status, LegacyAllowlistRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_legacy_allowlist_rollup("code-683", "label-439");
        if item.move_to(LegacyAllowlistRollupStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyAllowlistRollupStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_allowlist_rollup("code-683", "label-439");
        item.status = LegacyAllowlistRollupStatus::Closed;
        assert!(
            item.move_to(LegacyAllowlistRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
