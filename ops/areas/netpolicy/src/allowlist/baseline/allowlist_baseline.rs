use super::*;

use acme_kit::text;

/// Where a allowlist baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistBaselineStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AllowlistBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AllowlistBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBaseline {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub ceiling: f64,
    pub amount: f64,
    pub rotation_days: i64,
    pub floor_value: f64,
    pub status: AllowlistBaselineStatus,
}

impl AllowlistBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistBaselineStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistBaselineStatus) -> bool {
        match self.status {
            AllowlistBaselineStatus::Requested => {
                return next == AllowlistBaselineStatus::Revoked || next == AllowlistBaselineStatus::Expired;
            }
            AllowlistBaselineStatus::Granted => {
                match next {
                    AllowlistBaselineStatus::Revoked | AllowlistBaselineStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistBaselineStatus::Revoked => {
                match next {
                    AllowlistBaselineStatus::Expired => {
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

    pub fn move_to(&mut self, next: AllowlistBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.floor_value - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistBaseline {
        AllowlistBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            ceiling: self.ceiling,
            amount: self.amount,
            rotation_days: self.rotation_days,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a allowlist baseline in status requested.
pub fn new_allowlist_baseline(code: &str, label: &str) -> AllowlistBaseline {
    AllowlistBaseline {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        ceiling: 0.0,
        amount: 0.0,
        rotation_days: 0,
        floor_value: 0.0,
        status: AllowlistBaselineStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_allowlist_baseline("code-45", "label-412");
        assert_eq!(item.status, AllowlistBaselineStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_allowlist_baseline("code-45", "label-412");
        if item.move_to(AllowlistBaselineStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistBaselineStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_allowlist_baseline("code-45", "label-412");
        item.status = AllowlistBaselineStatus::Expired;
        assert!(
            item.move_to(AllowlistBaselineStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
