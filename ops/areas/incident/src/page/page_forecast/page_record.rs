use super::*;

use acme_kit::text;

/// Where a page record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PageRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PageRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageRecord {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub description: String,
    pub port: i64,
    pub audience: String,
    pub rotation_days: i64,
    pub cost: f64,
    pub updated_at: i64,
    pub channel: String,
    pub status: PageRecordStatus,
}

impl PageRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageRecordStatus) -> bool {
        match self.status {
            PageRecordStatus::Created => {
                return next == PageRecordStatus::Validated || next == PageRecordStatus::Published;
            }
            PageRecordStatus::Validated => {
                match next {
                    PageRecordStatus::Published | PageRecordStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageRecordStatus::Published => {
                return next == PageRecordStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page record {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.port as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageRecord {
        PageRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            description: self.description.clone(),
            port: self.port,
            audience: self.audience.clone(),
            rotation_days: self.rotation_days,
            cost: self.cost,
            updated_at: self.updated_at,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a page record in status created.
pub fn new_page_record(code: &str, name: &str) -> PageRecord {
    PageRecord {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        description: "".to_string(),
        port: 0,
        audience: "".to_string(),
        rotation_days: 0,
        cost: 0.0,
        updated_at: 0,
        channel: "".to_string(),
        status: PageRecordStatus::Created,
    }
}
