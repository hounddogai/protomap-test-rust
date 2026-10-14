use super::*;

use acme_kit::text;

/// Where a allowlist exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AllowlistExceptionStatus {
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

impl std::fmt::Display for AllowlistExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistException {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub unit: String,
    pub label: String,
    pub priority: i64,
    pub segment: String,
    pub minimum: f64,
    pub contact_email: String,
    pub status: AllowlistExceptionStatus,
}

impl AllowlistException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistExceptionStatus) -> bool {
        match self.status {
            AllowlistExceptionStatus::Draft => {
                match next {
                    AllowlistExceptionStatus::Suspended | AllowlistExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistExceptionStatus::Pending => {
                return next == AllowlistExceptionStatus::Suspended || next == AllowlistExceptionStatus::Closed;
            }
            AllowlistExceptionStatus::Active => {
                return next == AllowlistExceptionStatus::Closed;
            }
            AllowlistExceptionStatus::Suspended => {
                match next {
                    AllowlistExceptionStatus::Closed => {
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

    pub fn move_to(&mut self, next: AllowlistExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "allowlist exception", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.minimum + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistException {
        AllowlistException {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            unit: self.unit.clone(),
            label: self.label.clone(),
            priority: self.priority,
            segment: self.segment.clone(),
            minimum: self.minimum,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a allowlist exception in status draft.
pub fn new_allowlist_exception(code: &str, name: &str) -> AllowlistException {
    AllowlistException {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        unit: "".to_string(),
        label: "".to_string(),
        priority: 0,
        segment: "".to_string(),
        minimum: 0.0,
        contact_email: "".to_string(),
        status: AllowlistExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_allowlist_exception("code-95", "name-409");
        assert_eq!(item.status, AllowlistExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_allowlist_exception("code-95", "name-409");
        if item.move_to(AllowlistExceptionStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistExceptionStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_allowlist_exception("code-95", "name-409");
        item.status = AllowlistExceptionStatus::Closed;
        assert!(
            item.move_to(AllowlistExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
