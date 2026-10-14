use super::*;

/// How screens and exports show a probe quota.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeQuotaView {
    pub code: String,
    pub name: String,
    pub grace_days: String,
    pub ratio_text: String,
    pub quantity: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_probe_quota_view(item: &ProbeQuota) -> ProbeQuotaView {
    ProbeQuotaView {
        code: item.code.clone(),
        name: item.name.clone(),
        grace_days: item.grace_days.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        quantity: item.quantity.to_string(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_quota_views(items: &[ProbeQuota]) -> Vec<ProbeQuotaView> {
    let mut views: Vec<ProbeQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_quota_view(item));
    }
    views
}

pub fn sum_probe_quota_view(view: &ProbeQuotaView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_ratio + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_quota("code-463", "name-297");
        let view = to_probe_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
