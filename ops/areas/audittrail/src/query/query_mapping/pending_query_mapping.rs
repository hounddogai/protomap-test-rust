use super::*;

use acme_kit::text;

/// Where a pending query mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingQueryMappingStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingQueryMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PendingQueryMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending query mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMapping {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub title: String,
    pub audience: String,
    pub ends_at: i64,
    pub limit: i64,
    pub country: String,
    pub cluster: String,
    pub rate: f64,
    pub contact_email: String,
    pub status: PendingQueryMappingStatus,
}

impl PendingQueryMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingQueryMappingStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingQueryMappingStatus) -> bool {
        match self.status {
            PendingQueryMappingStatus::New => {
                match next {
                    PendingQueryMappingStatus::Running | PendingQueryMappingStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingQueryMappingStatus::Queued => {
                return next == PendingQueryMappingStatus::Running;
            }
            PendingQueryMappingStatus::Running => {
                return next == PendingQueryMappingStatus::Failed;
            }
            PendingQueryMappingStatus::Done => {
                return next == PendingQueryMappingStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingQueryMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending query mapping {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ends_at as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingQueryMapping {
        PendingQueryMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            ends_at: self.ends_at,
            limit: self.limit,
            country: self.country.clone(),
            cluster: self.cluster.clone(),
            rate: self.rate,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a pending query mapping in status new.
pub fn new_pending_query_mapping(code: &str, label: &str) -> PendingQueryMapping {
    PendingQueryMapping {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        title: "".to_string(),
        audience: "".to_string(),
        ends_at: 0,
        limit: 0,
        country: "".to_string(),
        cluster: "".to_string(),
        rate: 0.0,
        contact_email: "".to_string(),
        status: PendingQueryMappingStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_query_mapping("code-233", "label-796");
        assert_eq!(item.status, PendingQueryMappingStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_pending_query_mapping("code-233", "label-796");
        if item.move_to(PendingQueryMappingStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingQueryMappingStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_query_mapping("code-233", "label-796");
        item.status = PendingQueryMappingStatus::Failed;
        assert!(
            item.move_to(PendingQueryMappingStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
