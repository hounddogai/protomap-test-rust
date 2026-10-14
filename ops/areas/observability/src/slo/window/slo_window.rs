use super::*;

use acme_kit::text;

/// Where a slo window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SloWindowStatus {
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

impl std::fmt::Display for SloWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a slo window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SloWindow {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub minimum: f64,
    pub starts_at: i64,
    pub summary: String,
    pub version: i64,
    pub floor_value: f64,
    pub external_ref: String,
    pub host: String,
    pub checksum: String,
    pub status: SloWindowStatus,
}

impl SloWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloWindowStatus) -> bool {
        match self.status {
            SloWindowStatus::Draft => {
                match next {
                    SloWindowStatus::Active | SloWindowStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloWindowStatus::Pending => {
                return next == SloWindowStatus::Closed;
            }
            SloWindowStatus::Active => {
                return next == SloWindowStatus::Suspended || next == SloWindowStatus::Closed;
            }
            SloWindowStatus::Suspended => {
                return next == SloWindowStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "slo window", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.floor_value + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloWindow {
        SloWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            minimum: self.minimum,
            starts_at: self.starts_at,
            summary: self.summary.clone(),
            version: self.version,
            floor_value: self.floor_value,
            external_ref: self.external_ref.clone(),
            host: self.host.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a slo window in status draft.
pub fn new_slo_window(code: &str, name: &str) -> SloWindow {
    SloWindow {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        minimum: 0.0,
        starts_at: 0,
        summary: "".to_string(),
        version: 0,
        floor_value: 0.0,
        external_ref: "".to_string(),
        host: "".to_string(),
        checksum: "".to_string(),
        status: SloWindowStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_slo_window("code-982", "name-491");
        assert_eq!(item.status, SloWindowStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_slo_window("code-982", "name-491");
        if item.move_to(SloWindowStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloWindowStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_slo_window("code-982", "name-491");
        item.status = SloWindowStatus::Closed;
        assert!(item.move_to(SloWindowStatus::Draft).is_err(), "expected move to fail");
    }
}
