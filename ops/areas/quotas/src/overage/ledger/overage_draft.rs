use super::*;

use acme_kit::text;

/// Where a overage draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl OverageDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for OverageDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One overage draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageDraft {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub namespace: String,
    pub limit: i64,
    pub status: OverageDraftStatus,
}

impl OverageDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageDraftStatus) -> bool {
        match self.status {
            OverageDraftStatus::Requested => {
                return next == OverageDraftStatus::Granted || next == OverageDraftStatus::Revoked;
            }
            OverageDraftStatus::Granted => {
                match next {
                    OverageDraftStatus::Revoked | OverageDraftStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageDraftStatus::Revoked => {
                return next == OverageDraftStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "overage draft", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.margin + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageDraft {
        OverageDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            namespace: self.namespace.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a overage draft in status requested.
pub fn new_overage_draft(code: &str, title: &str) -> OverageDraft {
    OverageDraft {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        namespace: "".to_string(),
        limit: 0,
        status: OverageDraftStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_overage_draft("code-927", "title-565");
        assert_eq!(item.status, OverageDraftStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_overage_draft("code-927", "title-565");
        if item.move_to(OverageDraftStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageDraftStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_overage_draft("code-927", "title-565");
        item.status = OverageDraftStatus::Expired;
        assert!(
            item.move_to(OverageDraftStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
