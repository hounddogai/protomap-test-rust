use super::*;

use acme_kit::text;

/// Where a default nonce target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultNonceTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultNonceTargetStatus {
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

impl std::fmt::Display for DefaultNonceTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default nonce target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultNonceTarget {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub attempts: i64,
    pub floor_value: f64,
    pub external_ref: String,
    pub status: DefaultNonceTargetStatus,
}

impl DefaultNonceTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultNonceTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultNonceTargetStatus) -> bool {
        match self.status {
            DefaultNonceTargetStatus::Draft => {
                return next == DefaultNonceTargetStatus::Suspended || next == DefaultNonceTargetStatus::Closed;
            }
            DefaultNonceTargetStatus::Pending => {
                match next {
                    DefaultNonceTargetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultNonceTargetStatus::Active => {
                return next == DefaultNonceTargetStatus::Suspended;
            }
            DefaultNonceTargetStatus::Suspended => {
                return next == DefaultNonceTargetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultNonceTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default nonce target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.attempts as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultNonceTarget {
        DefaultNonceTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            attempts: self.attempts,
            floor_value: self.floor_value,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default nonce target in status draft.
pub fn new_default_nonce_target(code: &str, label: &str) -> DefaultNonceTarget {
    DefaultNonceTarget {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        attempts: 0,
        floor_value: 0.0,
        external_ref: "".to_string(),
        status: DefaultNonceTargetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_nonce_target("code-837", "label-345");
        assert_eq!(item.status, DefaultNonceTargetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_default_nonce_target("code-837", "label-345");
        if item.move_to(DefaultNonceTargetStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultNonceTargetStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_nonce_target("code-837", "label-345");
        item.status = DefaultNonceTargetStatus::Closed;
        assert!(
            item.move_to(DefaultNonceTargetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
