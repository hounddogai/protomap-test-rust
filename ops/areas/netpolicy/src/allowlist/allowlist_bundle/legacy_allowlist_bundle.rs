use super::*;

use acme_kit::text;

/// Where a legacy allowlist bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyAllowlistBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyAllowlistBundleStatus {
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

impl std::fmt::Display for LegacyAllowlistBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy allowlist bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBundle {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub sequence: i64,
    pub locale: String,
    pub capacity: i64,
    pub attempts: i64,
    pub ceiling: f64,
    pub status: LegacyAllowlistBundleStatus,
}

impl LegacyAllowlistBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyAllowlistBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyAllowlistBundleStatus) -> bool {
        match self.status {
            LegacyAllowlistBundleStatus::Draft => {
                return next == LegacyAllowlistBundleStatus::Active;
            }
            LegacyAllowlistBundleStatus::Pending => {
                match next {
                    LegacyAllowlistBundleStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistBundleStatus::Active => {
                match next {
                    LegacyAllowlistBundleStatus::Suspended | LegacyAllowlistBundleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistBundleStatus::Suspended => {
                match next {
                    LegacyAllowlistBundleStatus::Closed => {
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

    pub fn move_to(&mut self, next: LegacyAllowlistBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy allowlist bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.sequence as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyAllowlistBundle {
        LegacyAllowlistBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            sequence: self.sequence,
            locale: self.locale.clone(),
            capacity: self.capacity,
            attempts: self.attempts,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a legacy allowlist bundle in status draft.
pub fn new_legacy_allowlist_bundle(code: &str, name: &str) -> LegacyAllowlistBundle {
    LegacyAllowlistBundle {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        sequence: 0,
        locale: "".to_string(),
        capacity: 0,
        attempts: 0,
        ceiling: 0.0,
        status: LegacyAllowlistBundleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_allowlist_bundle("code-741", "name-92");
        assert_eq!(item.status, LegacyAllowlistBundleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_legacy_allowlist_bundle("code-741", "name-92");
        if item.move_to(LegacyAllowlistBundleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyAllowlistBundleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_allowlist_bundle("code-741", "name-92");
        item.status = LegacyAllowlistBundleStatus::Closed;
        assert!(
            item.move_to(LegacyAllowlistBundleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
