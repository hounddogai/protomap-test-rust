use super::*;

/// How screens and exports show a seasonal dns record reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordReservationView {
    pub code: String,
    pub name_text: String,
    pub channel: String,
    pub threshold_text: String,
    pub budget: String,
    pub port: String,
    pub contact_email: String,
    pub tenant_id: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_seasonal_dns_record_reservation_view(
    item: &SeasonalDnsRecordReservation,
) -> SeasonalDnsRecordReservationView {
    SeasonalDnsRecordReservationView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        channel: item.channel.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        budget: format!("{:.2}", item.budget),
        port: item.port.to_string(),
        contact_email: item.contact_email.clone(),
        tenant_id: item.tenant_id.clone(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_dns_record_reservation_views(
    items: &[SeasonalDnsRecordReservation],
) -> Vec<SeasonalDnsRecordReservationView> {
    let mut views: Vec<SeasonalDnsRecordReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_dns_record_reservation_view(item));
    }
    views
}

pub fn sum_seasonal_dns_record_reservation_view(view: &SeasonalDnsRecordReservationView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_threshold + parsed_budget + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_dns_record_reservation("code-395", "name-880");
        let view = to_seasonal_dns_record_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_dns_record_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
