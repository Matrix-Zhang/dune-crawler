use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
struct ExecuteQueryVariables {
    query_id: i32,
    parameters: Vec<Value>,
}

impl ExecuteQueryVariables {
    fn new(query_id: i32) -> Self {
        Self {
            query_id,
            parameters: vec![],
        }
    }
}

#[derive(Serialize)]
pub(crate) struct ExecuteQuery {
    #[serde(rename = "operationName")]
    operation_name: &'static str,
    variables: ExecuteQueryVariables,
    query: &'static str,
}

impl ExecuteQuery {
    pub(crate) fn new(query_id: i32) -> Self {
        Self {
            operation_name: "ExecuteQuery",
            variables: ExecuteQueryVariables::new(query_id),
            query: "mutation ExecuteQuery($query_id: Int!, $parameters: [Parameter!]!) {\n  execute_query(query_id: $query_id, parameters: $parameters) {\n    job_id\n    __typename\n  }\n}\n",
        }
    }
}

#[derive(Deserialize)]
struct ExecuteQueryResponseJob {
    job_id: String,
}

#[derive(Deserialize)]
struct ExecuteQueryResponseData {
    execute_query: ExecuteQueryResponseJob,
}

#[derive(Deserialize)]
pub(crate) struct ExecuteQueryResponse {
    data: ExecuteQueryResponseData,
}

impl ExecuteQueryResponse {
    pub(crate) fn job_id(self) -> String {
        self.data.execute_query.job_id
    }
}

#[test]
fn deserialize_execute_query_response_should_ok() {
    let json = r#"
    {
        "data": {
            "execute_query": {
                "job_id": "b0a5808d-a4f7-402e-b7e5-d6db9c0d0913",
                "__typename": "execute_query_response"
            }
        }
    }
    "#;

    let res = serde_json::from_str::<ExecuteQueryResponse>(&json).unwrap();
    assert_eq!(res.job_id(), "b0a5808d-a4f7-402e-b7e5-d6db9c0d0913");
}
