use super::*;

use acme_kit::text;

/// Where a allowlist window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistWindowStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AllowlistWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AllowlistWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist window as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistWindow {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub locale: String,
    pub cipher: String,
    pub weight: f64,
    pub status: AllowlistWindowStatus,
}

impl AllowlistWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistWindowStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistWindowStatus) -> bool {
        match self.status {
            AllowlistWindowStatus::Proposed => {
                match next {
                    AllowlistWindowStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistWindowStatus::Approved => {
                return next == AllowlistWindowStatus::Withdrawn;
            }
            AllowlistWindowStatus::Rejected => {
                return next == AllowlistWindowStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "allowlist window", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        (self.ratio * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistWindow {
        AllowlistWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            locale: self.locale.clone(),
            cipher: self.cipher.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a allowlist window in status proposed.
pub fn new_allowlist_window(code: &str, label: &str) -> AllowlistWindow {
    AllowlistWindow {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        locale: "".to_string(),
        cipher: "".to_string(),
        weight: 0.0,
        status: AllowlistWindowStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_allowlist_window("code-600", "label-333");
        assert_eq!(item.status, AllowlistWindowStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_allowlist_window("code-600", "label-333");
        if item.move_to(AllowlistWindowStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistWindowStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_allowlist_window("code-600", "label-333");
        item.status = AllowlistWindowStatus::Withdrawn;
        assert!(
            item.move_to(AllowlistWindowStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
