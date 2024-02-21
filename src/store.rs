use crate::data_models::Record;
use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::{Client, Collection, Database};
use tracing::info;

#[derive(Debug, Clone)]
pub struct MongoDB {
    client: Client,
}

impl MongoDB {
    pub async fn new(db_usr: &str, db_secred: &str) -> Result<Self, mongodb::error::Error> {
        let conn_str = format!("mongodb+srv://{db_usr}:{db_secred}@testcluster.mkc0xla.mongodb.net/?retryWrites=true&w=majority");
        info!("Establishing Database Connection.");
        let client = Client::with_uri_str(conn_str).await?;

        // Send a ping to confirm a successful connection
        // don't really, know whether this is something we should do, but otherwise we don't know
        // whether the credentials were right.
        info!("Created Database Client. Sending Ping, to ensure connection to the Database.");
        client
            .database("moneytag")
            .run_command(doc! { "ping": 1 }, None)
            .await?;
        info!("Ping was successfully. Database is available");

        Ok(MongoDB { client })
    }

    fn db(&self) -> Database {
        self.client.database("moneytag")
    }

    fn record_col(&self) -> Collection<Record> {
        self.db().collection("records")
    }

    pub async fn add_record(&self, record: Record) -> Result<(), mongodb::error::Error> {
        let rec_col = self.record_col();
        rec_col.insert_one(record, None).await?;
        Ok(())
    }

    pub async fn add_records(&self, records: Vec<Record>) -> Result<(), mongodb::error::Error> {
        let rec_col = self.record_col();
        rec_col.insert_many(records, None).await?;
        Ok(())
    }

    pub async fn records(&self, username: &str) -> Result<Vec<Record>, mongodb::error::Error> {
        let rec_col: Collection<Record> = self.db().collection("records");
        let cursor = rec_col.find(doc! { "owner": username}, None).await?;
        let result: Vec<_> = cursor.try_collect().await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::data_models::Record;
    use crate::store::MongoDB;

    const DB_SECRET: &str = "nothing";
    const DB_USER: &str = "some-name";

    #[tokio::test]
    async fn init_connection() {
        let init_result = MongoDB::new(DB_USER, DB_SECRET).await;

        assert!(
            init_result.is_ok(),
            "The database connection could not be established"
        );
    }

    #[tokio::test]
    async fn insert_retrieve_records() -> Result<(), mongodb::error::Error> {
        let db = MongoDB::new(DB_USER, DB_SECRET).await?;

        db.add_record(Record {
            owner: String::from("the owner"),
            title: String::from("test"),
            amount: 42.1415,
            tags: vec![String::from("test"), String::from("shared")],
        })
        .await?;

        let fetched_recs = db.records("the owner").await?;
        assert_eq!(1, fetched_recs.len());

        Ok(())
    }
}
