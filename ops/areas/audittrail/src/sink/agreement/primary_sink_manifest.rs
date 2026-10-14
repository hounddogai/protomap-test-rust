use super::*;

use acme_kit::text;

/// Where a primary sink manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySinkManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimarySinkManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PrimarySinkManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary sink manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkManifest {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub tags: Vec<String>,
    pub host: String,
    pub sequence: i64,
    pub namespace: String,
    pub external_ref: String,
    pub burst: i64,
    pub status: PrimarySinkManifestStatus,
}

impl PrimarySinkManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySinkManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySinkManifestStatus) -> bool {
        match self.status {
            PrimarySinkManifestStatus::Planned => {
                return next == PrimarySinkManifestStatus::Cancelled;
            }
            PrimarySinkManifestStatus::Scheduled => {
                match next {
                    PrimarySinkManifestStatus::InProgress | PrimarySinkManifestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySinkManifestStatus::InProgress => {
                match next {
                    PrimarySinkManifestStatus::Completed | PrimarySinkManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySinkManifestStatus::Completed => {
                match next {
                    PrimarySinkManifestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PrimarySinkManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary sink manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.burst as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySinkManifest {
        PrimarySinkManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            tags: self.tags.clone(),
            host: self.host.clone(),
            sequence: self.sequence,
            namespace: self.namespace.clone(),
            external_ref: self.external_ref.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a primary sink manifest in status planned.
pub fn new_primary_sink_manifest(code: &str, label: &str) -> PrimarySinkManifest {
    PrimarySinkManifest {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        tags: Vec::new(),
        host: "".to_string(),
        sequence: 0,
        namespace: "".to_string(),
        external_ref: "".to_string(),
        burst: 0,
        status: PrimarySinkManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_sink_manifest("code-891", "label-778");
        assert_eq!(item.status, PrimarySinkManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_primary_sink_manifest("code-891", "label-778");
        if item.move_to(PrimarySinkManifestStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySinkManifestStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_sink_manifest("code-891", "label-778");
        item.status = PrimarySinkManifestStatus::Cancelled;
        assert!(
            item.move_to(PrimarySinkManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
