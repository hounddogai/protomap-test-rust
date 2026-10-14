use super::*;

use acme_kit::text;

/// Where a quote cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteCycleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl QuoteCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for QuoteCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCycle {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub grace_days: i64,
    pub namespace: String,
    pub status: QuoteCycleStatus,
}

impl QuoteCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteCycleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteCycleStatus) -> bool {
        match self.status {
            QuoteCycleStatus::Requested => {
                return next == QuoteCycleStatus::Revoked || next == QuoteCycleStatus::Expired;
            }
            QuoteCycleStatus::Granted => {
                return next == QuoteCycleStatus::Revoked;
            }
            QuoteCycleStatus::Revoked => {
                match next {
                    QuoteCycleStatus::Expired => {
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

    pub fn move_to(&mut self, next: QuoteCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote cycle {} cannot move from {} to {}",
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
        self.grace_days as f64
    }

    pub fn copy(&self) -> QuoteCycle {
        QuoteCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            grace_days: self.grace_days,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a quote cycle in status requested.
pub fn new_quote_cycle(code: &str, label: &str) -> QuoteCycle {
    QuoteCycle {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        grace_days: 0,
        namespace: "".to_string(),
        status: QuoteCycleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_quote_cycle("code-889", "label-880");
        assert_eq!(item.status, QuoteCycleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_quote_cycle("code-889", "label-880");
        if item.move_to(QuoteCycleStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteCycleStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_quote_cycle("code-889", "label-880");
        item.status = QuoteCycleStatus::Expired;
        assert!(
            item.move_to(QuoteCycleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
