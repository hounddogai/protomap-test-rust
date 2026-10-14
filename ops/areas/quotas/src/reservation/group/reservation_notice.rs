use super::*;

use acme_kit::text;

/// Where a reservation notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationNoticeStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ReservationNoticeStatus {
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

impl std::fmt::Display for ReservationNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a reservation notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationNotice {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub contact_email: String,
    pub starts_at: i64,
    pub tenant_id: String,
    pub maximum: f64,
    pub threshold: f64,
    pub principal: String,
    pub limit: i64,
    pub host: String,
    pub status: ReservationNoticeStatus,
}

impl ReservationNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationNoticeStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationNoticeStatus) -> bool {
        match self.status {
            ReservationNoticeStatus::Draft => {
                match next {
                    ReservationNoticeStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationNoticeStatus::Pending => {
                match next {
                    ReservationNoticeStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationNoticeStatus::Active => {
                match next {
                    ReservationNoticeStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationNoticeStatus::Suspended => {
                match next {
                    ReservationNoticeStatus::Closed => {
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

    pub fn move_to(&mut self, next: ReservationNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "reservation notice", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.maximum * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationNotice {
        ReservationNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            contact_email: self.contact_email.clone(),
            starts_at: self.starts_at,
            tenant_id: self.tenant_id.clone(),
            maximum: self.maximum,
            threshold: self.threshold,
            principal: self.principal.clone(),
            limit: self.limit,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a reservation notice in status draft.
pub fn new_reservation_notice(code: &str, name: &str) -> ReservationNotice {
    ReservationNotice {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        contact_email: "".to_string(),
        starts_at: 0,
        tenant_id: "".to_string(),
        maximum: 0.0,
        threshold: 0.0,
        principal: "".to_string(),
        limit: 0,
        host: "".to_string(),
        status: ReservationNoticeStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_reservation_notice("code-298", "name-185");
        assert_eq!(item.status, ReservationNoticeStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_reservation_notice("code-298", "name-185");
        if item.move_to(ReservationNoticeStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationNoticeStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_reservation_notice("code-298", "name-185");
        item.status = ReservationNoticeStatus::Closed;
        assert!(
            item.move_to(ReservationNoticeStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
