use super::*;

use acme_kit::text;

/// Where a strict quote baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictQuoteBaselineStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictQuoteBaselineStatus {
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

impl std::fmt::Display for StrictQuoteBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict quote baseline as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictQuoteBaseline {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub ttl_seconds: i64,
    pub phone: String,
    pub discount: f64,
    pub ends_at: i64,
    pub weight: f64,
    pub cost: f64,
    pub description: String,
    pub rotation_days: i64,
    pub status: StrictQuoteBaselineStatus,
}

impl StrictQuoteBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictQuoteBaselineStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictQuoteBaselineStatus) -> bool {
        match self.status {
            StrictQuoteBaselineStatus::Draft => {
                match next {
                    StrictQuoteBaselineStatus::Suspended | StrictQuoteBaselineStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictQuoteBaselineStatus::Pending => {
                match next {
                    StrictQuoteBaselineStatus::Active | StrictQuoteBaselineStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictQuoteBaselineStatus::Active => {
                match next {
                    StrictQuoteBaselineStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictQuoteBaselineStatus::Suspended => {
                match next {
                    StrictQuoteBaselineStatus::Closed => {
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

    pub fn move_to(&mut self, next: StrictQuoteBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict quote baseline {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "strict quote baseline", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.weight * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictQuoteBaseline {
        StrictQuoteBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            ttl_seconds: self.ttl_seconds,
            phone: self.phone.clone(),
            discount: self.discount,
            ends_at: self.ends_at,
            weight: self.weight,
            cost: self.cost,
            description: self.description.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict quote baseline in status draft.
pub fn new_strict_quote_baseline(code: &str, title: &str) -> StrictQuoteBaseline {
    StrictQuoteBaseline {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        ttl_seconds: 0,
        phone: "".to_string(),
        discount: 0.0,
        ends_at: 0,
        weight: 0.0,
        cost: 0.0,
        description: "".to_string(),
        rotation_days: 0,
        status: StrictQuoteBaselineStatus::Draft,
    }
}
