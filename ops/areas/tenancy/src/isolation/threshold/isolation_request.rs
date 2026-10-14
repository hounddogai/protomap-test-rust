use super::*;

use acme_kit::text;

/// Where a isolation request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationRequestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IsolationRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IsolationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation request as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRequest {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub title: String,
    pub notes: String,
    pub quantity: i64,
    pub status: IsolationRequestStatus,
}

impl IsolationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationRequestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationRequestStatus) -> bool {
        match self.status {
            IsolationRequestStatus::Created => {
                match next {
                    IsolationRequestStatus::Validated | IsolationRequestStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationRequestStatus::Validated => {
                return next == IsolationRequestStatus::Published || next == IsolationRequestStatus::Retired;
            }
            IsolationRequestStatus::Published => {
                return next == IsolationRequestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> IsolationRequest {
        IsolationRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a isolation request in status created.
pub fn new_isolation_request(code: &str, label: &str) -> IsolationRequest {
    IsolationRequest {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        title: "".to_string(),
        notes: "".to_string(),
        quantity: 0,
        status: IsolationRequestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_isolation_request("code-751", "label-655");
        assert_eq!(item.status, IsolationRequestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_isolation_request("code-751", "label-655");
        if item.move_to(IsolationRequestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationRequestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_isolation_request("code-751", "label-655");
        item.status = IsolationRequestStatus::Retired;
        assert!(
            item.move_to(IsolationRequestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
