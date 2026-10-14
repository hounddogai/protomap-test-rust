use super::*;

use acme_kit::text;

/// Where a default timeline notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultTimelineNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultTimelineNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultTimelineNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default timeline notice as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineNotice {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub ttl_seconds: i64,
    pub tenant_id: String,
    pub external_ref: String,
    pub country: String,
    pub locale: String,
    pub updated_at: i64,
    pub status: DefaultTimelineNoticeStatus,
}

impl DefaultTimelineNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultTimelineNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultTimelineNoticeStatus) -> bool {
        match self.status {
            DefaultTimelineNoticeStatus::Proposed => {
                match next {
                    DefaultTimelineNoticeStatus::Approved | DefaultTimelineNoticeStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTimelineNoticeStatus::Approved => {
                match next {
                    DefaultTimelineNoticeStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTimelineNoticeStatus::Rejected => {
                match next {
                    DefaultTimelineNoticeStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DefaultTimelineNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default timeline notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultTimelineNotice {
        DefaultTimelineNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            ttl_seconds: self.ttl_seconds,
            tenant_id: self.tenant_id.clone(),
            external_ref: self.external_ref.clone(),
            country: self.country.clone(),
            locale: self.locale.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a default timeline notice in status proposed.
pub fn new_default_timeline_notice(code: &str, label: &str) -> DefaultTimelineNotice {
    DefaultTimelineNotice {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        ttl_seconds: 0,
        tenant_id: "".to_string(),
        external_ref: "".to_string(),
        country: "".to_string(),
        locale: "".to_string(),
        updated_at: 0,
        status: DefaultTimelineNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_timeline_notice("code-762", "label-59");
        assert_eq!(item.status, DefaultTimelineNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_default_timeline_notice("code-762", "label-59");
        if item.move_to(DefaultTimelineNoticeStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultTimelineNoticeStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_timeline_notice("code-762", "label-59");
        item.status = DefaultTimelineNoticeStatus::Withdrawn;
        assert!(
            item.move_to(DefaultTimelineNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
