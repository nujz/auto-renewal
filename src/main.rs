use tracing::{error, trace};

async fn do_renewal(url: &str, token: &str, cid: &str) -> anyhow::Result<()> {
    let mut map = std::collections::HashMap::new();
    map.insert("cid", cid);
    let _ = reqwest::Client::new()
        .post(url)
        .bearer_auth(token)
        .json(&map)
        .send()
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").unwrap();
    let pin_url = std::env::var("PIN_URL").unwrap();
    let pin_token = std::env::var("PIN_TOKEN").unwrap();
    let mut worker_sleep_seconds: u64 = {
        let secs = std::env::var("WORKER_SLEEP_SECS").unwrap();
        secs.parse().unwrap()
    };
    if worker_sleep_seconds < 10 {
        worker_sleep_seconds = 10;
    }

    let pool = sqlx::mysql::MySqlPool::connect(&database_url)
        .await
        .unwrap();

    loop {
        let limit: i32 = 50;

        let mut offset = 0;

        trace!("task start");
        loop {
            let recs = sqlx::query!(
                r#"
    select cid from data_user_file where mtype = 1 order by id limit ?,?
"#,
                offset,
                limit
            )
            .fetch_all(&pool)
            .await
            .unwrap();

            for rec in &recs {
                if rec.cid.is_some() {
                    let _cid = rec.cid.as_ref().unwrap();
                    if _cid.is_empty() {
                        continue;
                    }
                    let Ok(cid) = cid::Cid::try_from(_cid.trim().as_bytes()) else {
                        continue;
                    };

                    trace!("renewing {}", cid);

                    if let Err(err) =
                        do_renewal(&pin_url, &pin_token, cid.to_string().as_str()).await
                    {
                        error!("{}", err);
                    }
                }
            }

            if recs.len() < limit.try_into().unwrap() {
                break;
            }
            offset += limit;
        }

        trace!("task finished");

        tokio::time::sleep(std::time::Duration::from_secs(worker_sleep_seconds)).await;
    }
}
