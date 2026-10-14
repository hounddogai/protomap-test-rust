use super::*;

use acme_kit::text;

/// Where a pending redaction override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRedactionOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingRedactionOverrideStatus {
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

impl std::fmt::Display for PendingRedactionOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending redaction override as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionOverride {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub port: i64,
    pub discount: f64,
    pub description: String,
    pub host: String,
    pub threshold: f64,
    pub reviewer: String,
    pub score: f64,
    pub status: PendingRedactionOverrideStatus,
}

impl PendingRedactionOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRedactionOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRedactionOverrideStatus) -> bool {
        match self.status {
            PendingRedactionOverrideStatus::Draft => {
                return next == PendingRedactionOverrideStatus::Closed;
            }
            PendingRedactionOverrideStatus::Pending => {
                return next == PendingRedactionOverrideStatus::Active;
            }
            PendingRedactionOverrideStatus::Active => {
                match next {
                    PendingRedactionOverrideStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRedactionOverrideStatus::Suspended => {
                return next == PendingRedactionOverrideStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRedactionOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending redaction override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.port as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRedactionOverride {
        PendingRedactionOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            port: self.port,
            discount: self.discount,
            description: self.description.clone(),
            host: self.host.clone(),
            threshold: self.threshold,
            reviewer: self.reviewer.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending redaction override in status draft.
pub fn new_pending_redaction_override(code: &str, label: &str) -> PendingRedactionOverride {
    PendingRedactionOverride {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        port: 0,
        discount: 0.0,
        description: "".to_string(),
        host: "".to_string(),
        threshold: 0.0,
        reviewer: "".to_string(),
        score: 0.0,
        status: PendingRedactionOverrideStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_redaction_override("code-749", "label-236");
        assert_eq!(item.status, PendingRedactionOverrideStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_pending_redaction_override("code-749", "label-236");
        if item.move_to(PendingRedactionOverrideStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRedactionOverrideStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_redaction_override("code-749", "label-236");
        item.status = PendingRedactionOverrideStatus::Closed;
        assert!(
            item.move_to(PendingRedactionOverrideStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
