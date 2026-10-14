use super::*;

use acme_kit::text;

/// Where a default timeline draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultTimelineDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultTimelineDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultTimelineDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default timeline draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineDraft {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub rate: f64,
    pub quota_units: i64,
    pub source: String,
    pub status: DefaultTimelineDraftStatus,
}

impl DefaultTimelineDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultTimelineDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultTimelineDraftStatus) -> bool {
        match self.status {
            DefaultTimelineDraftStatus::Requested => {
                match next {
                    DefaultTimelineDraftStatus::Revoked | DefaultTimelineDraftStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTimelineDraftStatus::Granted => {
                match next {
                    DefaultTimelineDraftStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTimelineDraftStatus::Revoked => {
                return next == DefaultTimelineDraftStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultTimelineDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default timeline draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        (self.rate * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultTimelineDraft {
        DefaultTimelineDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            rate: self.rate,
            quota_units: self.quota_units,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a default timeline draft in status requested.
pub fn new_default_timeline_draft(code: &str, label: &str) -> DefaultTimelineDraft {
    DefaultTimelineDraft {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        rate: 0.0,
        quota_units: 0,
        source: "".to_string(),
        status: DefaultTimelineDraftStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_timeline_draft("code-835", "label-38");
        assert_eq!(item.status, DefaultTimelineDraftStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_default_timeline_draft("code-835", "label-38");
        if item.move_to(DefaultTimelineDraftStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultTimelineDraftStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_timeline_draft("code-835", "label-38");
        item.status = DefaultTimelineDraftStatus::Expired;
        assert!(
            item.move_to(DefaultTimelineDraftStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
