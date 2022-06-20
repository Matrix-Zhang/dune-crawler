use std::{
    fmt::{Debug, Formatter},
    task::{Context, Poll},
};

use futures_util::{future::BoxFuture, ready, stream::Stream, TryFutureExt};
use reqwest::Client;
use serde::Deserialize;

use crate::domain::*;

enum QueryTaskState {
    RefreshSession(Option<BoxFuture<'static, Result<SessionResponse, anyhow::Error>>>),
    UpsertQuery(BoxFuture<'static, Result<UpsertQueryResponse, anyhow::Error>>),
    ExecuteQuery(BoxFuture<'static, Result<String, anyhow::Error>>),
    GetQueuePosition(
        (
            String,
            Option<BoxFuture<'static, Result<String, anyhow::Error>>>,
        ),
    ),
    FindResult(BoxFuture<'static, Result<String, anyhow::Error>>),
}

#[derive(Deserialize)]
struct SessionResponse {
    token: String,
}

pub(crate) struct QueryTask {
    userid: i32,
    client: Client,
    label_type: String,
    label_name: String,
    base_address: Option<String>,
    bearer_token: String,
    amount: usize,
    state: QueryTaskState,
}

impl Debug for QueryTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type: {}, name: {}, amount: {}",
            self.label_type, self.label_name, self.amount
        )
    }
}

impl QueryTask {
    pub(crate) fn new(
        userid: i32,
        client: Client,
        label_type: String,
        label_name: String,
        amount: usize,
    ) -> Self {
        Self {
            client,
            userid,
            label_type,
            label_name,
            base_address: None,
            bearer_token: String::new(),
            amount,
            state: QueryTaskState::RefreshSession(None),
        }
    }
}

impl Stream for QueryTask {
    type Item = Result<Vec<AddressLabel>, anyhow::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.state {
            QueryTaskState::RefreshSession(ref mut fut) => match fut {
                Some(ref mut fut) => ready!(fut.as_mut().poll(cx))
                    .map(|response| {
                        self.bearer_token = response.token;
                        let limit = std::cmp::min(100000, self.amount);

                        let mut where_clause = format!(
                            "type = '{}' AND name = '{}' AND octet_length(address) > 0",
                            self.label_type, self.label_name
                        );

                        if let Some(ref base_address) = self.base_address {
                            where_clause.push_str(&format!("AND address > '{}'", base_address));
                        }

                        let query = format!("SELECT address, name AS label_name, type AS label_type FROM labels.labels WHERE {} ORDER BY address ASC LIMIT {}", where_clause, limit);

                        //TODO: user_id
                        let fut = self
                            .client
                            .post("https://core-hsr.duneanalytics.com/v1/graphql")
                            .bearer_auth(&self.bearer_token)
                            .json(&UpsertQuery::new(self.userid, self.userid, query))
                            .send()
                            .and_then(|response| response.json::<UpsertQueryResponse>())
                            .map_err(Into::into);

                        self.state = QueryTaskState::UpsertQuery(Box::pin(fut));
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    })
                    .unwrap_or_else(|err| Poll::Ready(Some(Err(err)))),
                None => {
                    if self.amount == 0 {
                        return Poll::Ready(None);
                    }

                    let fut = self
                        .client
                        .post("https://dune.com/api/auth/session")
                        .send()
                        .and_then(|response| response.json::<SessionResponse>())
                        .map_err(Into::into);

                    self.state = QueryTaskState::RefreshSession(Some(Box::pin(fut)));
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
            QueryTaskState::UpsertQuery(ref mut fut) => ready!(fut.as_mut().poll(cx))
                .map(|response| {
                    let fut = self
                        .client
                        .post("https://core-hsr.duneanalytics.com/v1/graphql")
                        .bearer_auth(&self.bearer_token)
                        .json(&crate::domain::ExecuteQuery::new(response.query_id()))
                        .send()
                        .and_then(|response| response.text())
                        .map_err(Into::into);

                    self.state = QueryTaskState::ExecuteQuery(Box::pin(fut));
                    cx.waker().wake_by_ref();
                    Poll::Pending
                })
                .unwrap_or_else(|err| Poll::Ready(Some(Err(err)))),
            QueryTaskState::ExecuteQuery(ref mut fut) => {
                ready!(fut.as_mut().poll(cx).map_ok(|res| {
                    match serde_json::from_str::<ExecuteQueryResponse>(&res) {
                        Ok(json) => json,
                        Err(err) => panic!("{}, {}", res, err),
                    }
                }))
                .map(|response| {
                    self.state = QueryTaskState::GetQueuePosition((response.job_id(), None));
                    cx.waker().wake_by_ref();
                    Poll::Pending
                })
                .unwrap_or_else(|err| Poll::Ready(Some(Err(err))))
            }
            QueryTaskState::GetQueuePosition((ref job_id, ref mut fut)) => {
                let job_id = job_id.to_string();

                if let Some(ref mut fut) = fut {
                    match ready!(
                        fut.as_mut().poll(cx).map_ok(|res| {
                        match serde_json::from_str::<GetQueuePositionResponse>(&res) {
                            Ok(json) => json,
                            Err(err) => panic!("{}, {}", res, err),
                        }
                    })
                    ) {
                        Ok(response) => {
                            if !response.is_executing() {
                                let fut = self
                                    .client
                                    .post("https://core-hsr.duneanalytics.com/v1/graphql")
                                    .bearer_auth(&self.bearer_token)
                                    .json(&FindResultDataByJobId::new(job_id))
                                    .send()
                                    .and_then(|response| response.text())
                                    .map_err(Into::into);

                                self.state = QueryTaskState::FindResult(Box::pin(fut));
                                cx.waker().wake_by_ref();
                                return Poll::Pending;
                            }
                        }
                        Err(err) => return Poll::Ready(Some(Err(err))),
                    }
                }

                let fut = self
                    .client
                    .post("https://core-hsr.duneanalytics.com/v1/graphql")
                    .bearer_auth(&self.bearer_token)
                    .json(&crate::domain::GetQueuePosition::new(job_id.clone()))
                    .send()
                    .and_then(|response| response.text())
                    .map_err(Into::into);

                self.state = QueryTaskState::GetQueuePosition((job_id, Some(Box::pin(fut))));
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            QueryTaskState::FindResult(ref mut fut) => ready!(
                fut.as_mut().poll(cx).map_ok(|res| {
                    match serde_json::from_str::<FindResultDataResponse<AddressLabel>>(&res) {
                        Ok(json) => json.data(),
                        Err(err) => panic!("{},{}", res, err),
                    }
                })
            ).map(|data| {
                if let Some(last) = data.last() {
                    self.amount -= data.len();
                    self.base_address = Some(last.address().to_owned());
                    self.state = QueryTaskState::RefreshSession(None);
                    Poll::Ready(Some(Ok(data)))
                } else {
                    Poll::Ready(None)
                }
            }).unwrap_or_else(|err| Poll::Ready(Some(Err(err)))),
        }
    }
}
