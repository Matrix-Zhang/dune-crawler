use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GetQueuePositionVariables {
    job_id: String,
}

impl GetQueuePositionVariables {
    fn new(job_id: String) -> Self {
        Self { job_id }
    }
}

#[derive(Serialize)]
pub(crate) struct GetQueuePosition {
    #[serde(rename = "operationName")]
    operation_name: &'static str,
    variables: GetQueuePositionVariables,
    query: &'static str,
}

impl GetQueuePosition {
    pub(crate) fn new(job_id: String) -> Self {
        Self {
            operation_name: "GetQueuePosition",
            variables: GetQueuePositionVariables::new(job_id),
            query: "query GetQueuePosition($job_id: uuid!) {\n  view_queue_positions(where: {id: {_eq: $job_id}}) {\n    pos\n    __typename\n  }\n  jobs_by_pk(id: $job_id) {\n    id\n    user_id\n    category\n    created_at\n    locked_until\n    __typename\n  }\n}\n",
        }
    }
}

#[derive(Debug, Deserialize)]
struct GetQueuePositionResponseJob {
    category: String,
}

#[derive(Debug, Deserialize)]
struct GetQueuePositionResponseData {
    jobs_by_pk: Option<GetQueuePositionResponseJob>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GetQueuePositionResponse {
    data: GetQueuePositionResponseData,
}

impl GetQueuePositionResponse {
    pub(crate) fn is_executing(&self) -> bool {
        self.data
            .jobs_by_pk
            .as_ref()
            .map(|job| job.category.as_str())
            == Some("execute")
    }
}

#[test]
fn deserialize_get_query_position_should_ok() {
    let json = r#"
    {
        "data": {
          "view_queue_positions": [],
          "jobs_by_pk": {
            "id": "ae3b0c14-4022-426b-87ab-82048135d22c",
            "user_id": 121830,
            "category": "execute",
            "created_at": "2022-06-16T13:01:04.593305+00:00",
            "locked_until": "2022-06-16T13:31:04.594428+00:00",
            "__typename": "jobs"
          }
        }
      }
    "#;

    let res = serde_json::from_str::<GetQueuePositionResponse>(&json).unwrap();
    assert!(res.data.jobs_by_pk.is_some());
    assert!(res.is_executing());
}
