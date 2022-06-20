use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize)]
struct FindResultDataByJobIdVariables {
    job_id: String,
}

impl FindResultDataByJobIdVariables {
    fn new(job_id: String) -> Self {
        Self { job_id }
    }
}

#[derive(Serialize)]
pub(crate) struct FindResultDataByJobId {
    #[serde(rename = "operationName")]
    operation_name: &'static str,
    variables: FindResultDataByJobIdVariables,
    query: &'static str,
}

impl FindResultDataByJobId {
    pub(crate) fn new(job_id: String) -> Self {
        Self {
            operation_name: "FindResultDataByJob",
            variables: FindResultDataByJobIdVariables::new(job_id),
            query: "query FindResultDataByJob($job_id: uuid!) {\n  query_results(where: {job_id: {_eq: $job_id}, error: {_is_null: true}}) {\n    id\n    job_id\n    runtime\n    generated_at\n    columns\n    __typename\n  }\n  query_errors(where: {job_id: {_eq: $job_id}}) {\n    id\n    job_id\n    runtime\n    message\n    metadata\n    type\n    generated_at\n    __typename\n  }\n  get_result_by_job_id(args: {want_job_id: $job_id}) {\n    data\n    __typename\n  }\n}\n",
        }
    }
}

#[derive(Serialize)]
struct FindResultDataByResultIdVariables {
    result_id: String,
    error_id: &'static str,
}

impl FindResultDataByResultIdVariables {
    fn new(result_id: String) -> Self {
        Self {
            result_id,
            error_id: "00000000-0000-0000-0000-000000000000",
        }
    }
}

#[derive(Serialize)]
pub(crate) struct FindResultDataByResultId {
    #[serde(rename = "operationName")]
    operation_name: &'static str,
    variables: FindResultDataByResultIdVariables,
    query: &'static str,
}

impl FindResultDataByResultId {
    pub(crate) fn new(result_id: String) -> Self {
        Self {
            operation_name: "FindResultDataByResult",
            variables: FindResultDataByResultIdVariables::new(result_id),
            query: "query FindResultDataByResult($result_id: uuid!, $error_id: uuid!) {\n  query_results(where: {id: {_eq: $result_id}}) {\n    id\n    job_id\n    runtime\n    generated_at\n    columns\n    __typename\n  }\n  query_errors(where: {id: {_eq: $error_id}}) {\n    id\n    job_id\n    runtime\n    message\n    metadata\n    type\n    generated_at\n    __typename\n  }\n  get_result_by_result_id(args: {want_result_id: $result_id}) {\n    data\n    __typename\n  }\n}\n",
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct FindResult<T> {
    data: T,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FindResultsData<T> {
    ByJob {
        get_result_by_job_id: Vec<FindResult<T>>,
    },
    ByResult {
        get_result_by_result_id: Vec<FindResult<T>>,
    },
}

#[derive(Deserialize)]
pub(crate) struct FindResultDataResponse<T> {
    data: FindResultsData<T>,
}

impl<T> FindResultDataResponse<T>
where
    T: DeserializeOwned,
{
    pub(crate) fn data(self) -> Vec<T> {
        match self.data {
            FindResultsData::ByJob {
                get_result_by_job_id,
            } => get_result_by_job_id
                .into_iter()
                .map(|result| result.data)
                .collect(),
            FindResultsData::ByResult {
                get_result_by_result_id,
            } => get_result_by_result_id
                .into_iter()
                .map(|result| result.data)
                .collect(),
        }
    }
}

#[test]
fn deserialize_find_result_data_response_should_ok() {
    use serde_json::Value;

    let json = r#"
    {
        "data": {
          "query_results": [],
          "get_result_by_result_id": []
        }
      }
    "#;

    assert!(serde_json::from_str::<FindResultDataResponse<Value>>(&json).is_ok());
}
