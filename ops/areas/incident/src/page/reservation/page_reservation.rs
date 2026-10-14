use super::*;

use acme_kit::text;

/// Where a page reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PageReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PageReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page reservation as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageReservation {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub active: bool,
    pub updated_at: i64,
    pub status: PageReservationStatus,
}

impl PageReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageReservationStatus) -> bool {
        match self.status {
            PageReservationStatus::Requested => {
                match next {
                    PageReservationStatus::Granted | PageReservationStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageReservationStatus::Granted => {
                return next == PageReservationStatus::Expired;
            }
            PageReservationStatus::Revoked => {
                match next {
                    PageReservationStatus::Expired => {
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

    pub fn move_to(&mut self, next: PageReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page reservation {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "page reservation", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.updated_at as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageReservation {
        PageReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            active: self.active,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a page reservation in status requested.
pub fn new_page_reservation(code: &str, title: &str) -> PageReservation {
    PageReservation {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        active: false,
        updated_at: 0,
        status: PageReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_page_reservation("code-944", "title-924");
        assert_eq!(item.status, PageReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_page_reservation("code-944", "title-924");
        if item.move_to(PageReservationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageReservationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_page_reservation("code-944", "title-924");
        item.status = PageReservationStatus::Expired;
        assert!(
            item.move_to(PageReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
