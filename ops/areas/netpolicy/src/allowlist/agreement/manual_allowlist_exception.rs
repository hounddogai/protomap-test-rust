use super::*;

use acme_kit::text;

/// Where a manual allowlist exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualAllowlistExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualAllowlistExceptionStatus {
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

impl std::fmt::Display for ManualAllowlistExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual allowlist exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistException {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub rotation_days: i64,
    pub unit: String,
    pub reviewer: String,
    pub cipher: String,
    pub status: ManualAllowlistExceptionStatus,
}

impl ManualAllowlistException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualAllowlistExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualAllowlistExceptionStatus) -> bool {
        match self.status {
            ManualAllowlistExceptionStatus::Draft => {
                return next == ManualAllowlistExceptionStatus::Active
                    || next == ManualAllowlistExceptionStatus::Closed;
            }
            ManualAllowlistExceptionStatus::Pending => {
                return next == ManualAllowlistExceptionStatus::Suspended
                    || next == ManualAllowlistExceptionStatus::Closed;
            }
            ManualAllowlistExceptionStatus::Active => {
                match next {
                    ManualAllowlistExceptionStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualAllowlistExceptionStatus::Suspended => {
                match next {
                    ManualAllowlistExceptionStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualAllowlistExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual allowlist exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.version as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualAllowlistException {
        ManualAllowlistException {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            rotation_days: self.rotation_days,
            unit: self.unit.clone(),
            reviewer: self.reviewer.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a manual allowlist exception in status draft.
pub fn new_manual_allowlist_exception(code: &str, title: &str) -> ManualAllowlistException {
    ManualAllowlistException {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        rotation_days: 0,
        unit: "".to_string(),
        reviewer: "".to_string(),
        cipher: "".to_string(),
        status: ManualAllowlistExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_allowlist_exception("code-452", "title-508");
        assert_eq!(item.status, ManualAllowlistExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_manual_allowlist_exception("code-452", "title-508");
        if item.move_to(ManualAllowlistExceptionStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualAllowlistExceptionStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_allowlist_exception("code-452", "title-508");
        item.status = ManualAllowlistExceptionStatus::Closed;
        assert!(
            item.move_to(ManualAllowlistExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
