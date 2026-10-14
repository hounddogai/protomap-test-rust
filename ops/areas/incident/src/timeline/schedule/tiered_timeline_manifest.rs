use super::*;

use acme_kit::text;

/// Where a tiered timeline manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredTimelineManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredTimelineManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredTimelineManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered timeline manifest as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineManifest {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub ratio: f64,
    pub namespace: String,
    pub contact_email: String,
    pub maximum: f64,
    pub title: String,
    pub attempts: i64,
    pub status: TieredTimelineManifestStatus,
}

impl TieredTimelineManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredTimelineManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredTimelineManifestStatus) -> bool {
        match self.status {
            TieredTimelineManifestStatus::Created => {
                match next {
                    TieredTimelineManifestStatus::Validated | TieredTimelineManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredTimelineManifestStatus::Validated => {
                return next == TieredTimelineManifestStatus::Published
                    || next == TieredTimelineManifestStatus::Retired;
            }
            TieredTimelineManifestStatus::Published => {
                match next {
                    TieredTimelineManifestStatus::Retired => {
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

    pub fn move_to(&mut self, next: TieredTimelineManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered timeline manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.attempts as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredTimelineManifest {
        TieredTimelineManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            ratio: self.ratio,
            namespace: self.namespace.clone(),
            contact_email: self.contact_email.clone(),
            maximum: self.maximum,
            title: self.title.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a tiered timeline manifest in status created.
pub fn new_tiered_timeline_manifest(code: &str, label: &str) -> TieredTimelineManifest {
    TieredTimelineManifest {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        ratio: 0.0,
        namespace: "".to_string(),
        contact_email: "".to_string(),
        maximum: 0.0,
        title: "".to_string(),
        attempts: 0,
        status: TieredTimelineManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_timeline_manifest("code-594", "label-111");
        assert_eq!(item.status, TieredTimelineManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tiered_timeline_manifest("code-594", "label-111");
        if item.move_to(TieredTimelineManifestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredTimelineManifestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_timeline_manifest("code-594", "label-111");
        item.status = TieredTimelineManifestStatus::Retired;
        assert!(
            item.move_to(TieredTimelineManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
