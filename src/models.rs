#[derive(Debug, toasty::Model)]
pub(crate) struct Incident {
    #[key]
    #[auto]
    pub(crate) id: i64,
    pub(crate) public_id: String,
    pub(crate) title: String,
    pub(crate) summary: String,
}

#[derive(Debug, toasty::Model)]
pub(crate) struct IncidentUpdate {
    #[key]
    #[auto]
    pub(crate) id: i64,
    pub(crate) public_id: String,
    pub(crate) incident_id: i64,
    pub(crate) status: String,
    pub(crate) message: String,
}

#[cfg(test)]
mod tests {
    use super::{Incident, IncidentUpdate};
    use toasty::Db;

    #[tokio::test]
    async fn incidents_and_updates_round_trip_through_sqlite() {
        let mut db = Db::builder()
            .models(toasty::models!(crate::*))
            .connect("sqlite::memory:")
            .await
            .unwrap();
        db.push_schema().await.unwrap();

        let incident = toasty::create!(Incident {
            public_id: uuid::Uuid::now_v7().to_string(),
            title: "search is slow".to_owned(),
            summary: "Some queries are delayed.".to_owned(),
        })
        .exec(&mut db)
        .await
        .unwrap();
        toasty::create!(IncidentUpdate {
            incident_id: incident.id,
            public_id: uuid::Uuid::now_v7().to_string(),
            status: "Monitoring".to_owned(),
            message: "A fix is in place.".to_owned(),
        })
        .exec(&mut db)
        .await
        .unwrap();

        let saved_incident = Incident::filter_by_id(incident.id)
            .first()
            .exec(&mut db)
            .await
            .unwrap()
            .unwrap();
        let saved_updates = IncidentUpdate::all().exec(&mut db).await.unwrap();
        assert_eq!(saved_incident.title, "search is slow");
        assert_eq!(saved_incident.public_id.as_bytes()[14], b'7');
        assert_eq!(saved_updates.len(), 1);
        assert_eq!(saved_updates[0].message, "A fix is in place.");
        assert_eq!(saved_updates[0].public_id.as_bytes()[14], b'7');
    }
}
