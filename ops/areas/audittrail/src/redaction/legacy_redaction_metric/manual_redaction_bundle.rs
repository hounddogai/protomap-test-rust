use super::*;

use acme_kit::text;

/// Where a manual redaction bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRedactionBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualRedactionBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualRedactionBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual redaction bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionBundle {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub quantity: i64,
    pub weight: f64,
    pub limit: i64,
    pub ratio: f64,
    pub burst: i64,
    pub summary: String,
    pub score: f64,
    pub priority: i64,
    pub status: ManualRedactionBundleStatus,
}

impl ManualRedactionBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRedactionBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRedactionBundleStatus) -> bool {
        match self.status {
            ManualRedactionBundleStatus::Requested => {
                return next == ManualRedactionBundleStatus::Revoked || next == ManualRedactionBundleStatus::Expired;
            }
            ManualRedactionBundleStatus::Granted => {
                match next {
                    ManualRedactionBundleStatus::Revoked | ManualRedactionBundleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRedactionBundleStatus::Revoked => {
                match next {
                    ManualRedactionBundleStatus::Expired => {
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

    pub fn move_to(&mut self, next: ManualRedactionBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual redaction bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual redaction bundle", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.priority as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRedactionBundle {
        ManualRedactionBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            quantity: self.quantity,
            weight: self.weight,
            limit: self.limit,
            ratio: self.ratio,
            burst: self.burst,
            summary: self.summary.clone(),
            score: self.score,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual redaction bundle in status requested.
pub fn new_manual_redaction_bundle(code: &str, label: &str) -> ManualRedactionBundle {
    ManualRedactionBundle {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        quantity: 0,
        weight: 0.0,
        limit: 0,
        ratio: 0.0,
        burst: 0,
        summary: "".to_string(),
        score: 0.0,
        priority: 0,
        status: ManualRedactionBundleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_redaction_bundle("code-958", "label-130");
        assert_eq!(item.status, ManualRedactionBundleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_manual_redaction_bundle("code-958", "label-130");
        if item.move_to(ManualRedactionBundleStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRedactionBundleStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_redaction_bundle("code-958", "label-130");
        item.status = ManualRedactionBundleStatus::Expired;
        assert!(
            item.move_to(ManualRedactionBundleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
