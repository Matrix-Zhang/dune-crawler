use futures_util::{stream::select_all, StreamExt, TryFutureExt};
use rayon::prelude::*;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::sync::mpsc;

mod configuration;
mod domain;
mod query_task;

use configuration::*;
use domain::*;
use query_task::*;

const QUERIES_COUNT_ONE_TIME: usize = 2;

fn get_connection_pool(settings: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(settings.with_db())
}

#[derive(Deserialize)]
struct LabelData {
    label_name: String,
    label_type: String,
    amount: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::new()?;

    let db_pool = get_connection_pool(&settings.database());

    sqlx::migrate!().run(&db_pool).await.unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        "cookie",
        HeaderValue::from_str(settings.application().cookie())?,
    );

    let client = Client::builder().default_headers(headers).build().unwrap();

    let (tx, mut rx) = mpsc::channel(QUERIES_COUNT_ONE_TIME);

    let res = Client::new()
        .post("https://core-hsr.duneanalytics.com/v1/graphql")
        .json(&FindResultDataByResultId::new(String::from(
            "887c3f39-89cb-4f1b-92fc-98c22dc02f2b",
        )))
        .send()
        .and_then(|response| response.json::<FindResultDataResponse<LabelData>>())
        .await?;

    let shared_tx = tx.clone();
    tokio::spawn(async move {
        for data in res
            .data()
            .into_par_iter()
            .filter(|data| {
                data.amount > 1
                    && !data.label_type.starts_with("ens")
                    && !data.label_type.contains("contract")
            })
            .collect::<Vec<LabelData>>()
        {
            if let Err(err) = shared_tx
                .clone()
                .send(QueryTask::new(
                    settings.application().userid(),
                    client.clone(),
                    data.label_type,
                    data.label_name,
                    data.amount,
                ))
                .await
            {
                eprintln!("failed to send a new query, err: {}", err);
            }
        }
    });

    std::mem::drop(tx);

    let mut exit = false;
    let mut tasks = Vec::new();

    loop {
        match rx.recv().await {
            Some(task) => {
                tasks.push(task);
            }
            _ => {
                exit = true;
            }
        }

        if tasks.len() >= QUERIES_COUNT_ONE_TIME || exit {
            while let Some(res) = select_all(tasks.drain(0..)).next().await {
                match res {
                    Ok(data) => match db_pool.begin().await {
                        Ok(mut transaction) => {
                            let count = data.len();
                            for record in data {
                                if let Err(err) = sqlx::query!("INSERT INTO dune_labels(address, label_type, label_name) VALUES($1, $2, $3)",
                                    record.address(),
                                    record.label_type(),
                                    record.label_name()
                                )
                                    .execute(&mut transaction)
                                    .await
                                {
                                    println!("failed to insert new record, {}", err);
                                }
                            }
                            transaction.commit().await.unwrap();
                            println!("new labels, count: {}", count);
                        }
                        Err(err) => panic!("failed to get transaction, {:?}", err),
                    },
                    Err(err) => {
                        println!("err: {}", err);
                    }
                }
            }
        }

        if exit {
            break;
        }
    }

    Ok(())
}
