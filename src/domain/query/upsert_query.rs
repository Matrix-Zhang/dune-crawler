use serde::{Deserialize, Serialize};

use serde_json::{Map, Value};

#[derive(Serialize)]
struct UpsertQueryVisualizationsData {
    r#type: &'static str,
    name: &'static str,
    options: Value,
}

impl Default for UpsertQueryVisualizationsData {
    fn default() -> Self {
        Self {
            r#type: "table",
            name: "Query results",
            options: Value::Object(Map::new()),
        }
    }
}

#[derive(Serialize)]
struct UpsertQueryVisualizationsOnConfilct {
    constraint: &'static str,
    update_columns: Vec<Value>,
}

impl Default for UpsertQueryVisualizationsOnConfilct {
    fn default() -> Self {
        Self {
            constraint: "visualizations_pkey",
            update_columns: vec![],
        }
    }
}

#[derive(Serialize)]
struct UpsertQueryVisualizations {
    data: Vec<UpsertQueryVisualizationsData>,
    on_conflict: UpsertQueryVisualizationsOnConfilct,
}

impl Default for UpsertQueryVisualizations {
    fn default() -> Self {
        Self {
            data: vec![UpsertQueryVisualizationsData::default()],
            on_conflict: Default::default(),
        }
    }
}

#[derive(Serialize)]
struct UpsertQueryVariablesObject {
    schedule: Option<Value>,
    dataset_id: i32,
    name: &'static str,
    query: String,
    user_id: i32,
    team_id: Option<Value>,
    description: &'static str,
    is_archived: bool,
    is_temp: bool,
    is_private: bool,
    parameters: Vec<Value>,
    visualizations: UpsertQueryVisualizations,
}

impl UpsertQueryVariablesObject {
    fn new(user_id: i32, query: String) -> Self {
        Self {
            user_id,
            query,
            schedule: None,
            dataset_id: 4,
            name: "NewQuery",
            team_id: None,
            description: "",
            is_archived: false,
            is_temp: true,
            is_private: false,
            parameters: vec![],
            visualizations: UpsertQueryVisualizations::default(),
        }
    }
}

#[derive(Serialize)]
struct UpsertQueryOnConflict {
    constraint: String,
    update_columns: Vec<Value>,
}

impl Default for UpsertQueryOnConflict {
    fn default() -> Self {
        Self {
            constraint: String::from("queries_pkey"),
            update_columns: vec![],
        }
    }
}

#[derive(Serialize)]
struct UpsertQueryVariables {
    favs_last_24h: bool,
    favs_last_7d: bool,
    favs_last_30d: bool,
    favs_all_time: bool,
    object: UpsertQueryVariablesObject,
    on_conflict: UpsertQueryOnConflict,
    session_id: i32,
}

impl UpsertQueryVariables {
    fn new(user_id: i32, session_id: i32, query: String) -> Self {
        Self {
            session_id,
            favs_last_24h: false,
            favs_last_7d: false,
            favs_last_30d: false,
            favs_all_time: true,
            object: UpsertQueryVariablesObject::new(user_id, query),
            on_conflict: UpsertQueryOnConflict::default(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct UpsertQuery {
    #[serde(rename = "operationName")]
    operation_name: String,
    variables: UpsertQueryVariables,
    query: &'static str,
}

impl UpsertQuery {
    pub(crate) fn new(user_id: i32, session_id: i32, query: String) -> Self {
        Self {
            operation_name: String::from("UpsertQuery"),
            variables: UpsertQueryVariables::new(user_id, session_id, query),
            query: r#"mutation UpsertQuery($session_id: Int!, $object: queries_insert_input!, $on_conflict: queries_on_conflict!, $favs_last_24h: Boolean! = false, $favs_last_7d: Boolean! = false, $favs_last_30d: Boolean! = false, $favs_all_time: Boolean! = true) {  insert_queries_one(object: $object, on_conflict: $on_conflict) {    ...Query    favorite_queries(where: {user_id: {_eq: $session_id}}, limit: 1) {      created_at      __typename    }    __typename  }}fragment Query on queries {  ...BaseQuery  ...QueryVisualizations  ...QueryForked  ...QueryUsers  ...QueryTeams  ...QueryFavorites  __typename}fragment BaseQuery on queries {  id  dataset_id  name  description  query  is_private  is_temp  is_archived  created_at  updated_at  schedule  tags  parameters  __typename}fragment QueryVisualizations on queries {  visualizations {    id    type    name    options    created_at    __typename  }  __typename}fragment QueryForked on queries {  forked_query {    id    name    user {      name      __typename    }    team {      handle      __typename    }    __typename  }  __typename}fragment QueryUsers on queries {  user {    ...User    __typename  }  team {    id    name    handle    profile_image_url    __typename  }  __typename}fragment User on users {  id  name  profile_image_url  __typename}fragment QueryTeams on queries {  team {    ...Team    __typename  }  __typename}fragment Team on teams {  id  name  handle  profile_image_url  __typename}fragment QueryFavorites on queries {  query_favorite_count_all @include(if: $favs_all_time) {    favorite_count    __typename  }  query_favorite_count_last_24h @include(if: $favs_last_24h) {    favorite_count    __typename  }  query_favorite_count_last_7d @include(if: $favs_last_7d) {    favorite_count    __typename  }  query_favorite_count_last_30d @include(if: $favs_last_30d) {    favorite_count    __typename  }  __typename}"#,
        }
    }
}

#[derive(Deserialize)]
struct UpsertQueryResponseDataInsertQueriesOne {
    #[serde(rename = "id")]
    query_id: i32,
}

#[derive(Deserialize)]
struct UpsertQueryResponseData {
    insert_queries_one: UpsertQueryResponseDataInsertQueriesOne,
}

#[derive(Deserialize)]
pub(crate) struct UpsertQueryResponse {
    data: UpsertQueryResponseData,
}

impl UpsertQueryResponse {
    pub(crate) fn query_id(&self) -> i32 {
        self.data.insert_queries_one.query_id
    }
}

#[test]
fn deserialize_upsert_query_response_should_ok() {
    let json = r#"
    {
        "data": {
            "insert_queries_one": {
                "id": 917003,
                "dataset_id": 4,
                "name": "New Query",
                "description": "",
                "query": "SELECT\n  address,\n  \"name\" as label_name,\n  \"type\" as label_type\nFROM\n  labels.labels\nWHERE\n  name = 'dex trader'\n  AND type = 'activity'\n  AND octet_length(address) > 0\nORDER BY\n  address ASC\nLIMIT\n  10",
                "is_private": false,
                "is_temp": true,
                "is_archived": false,
                "created_at": "2022-06-17T01:46:01.596472+00:00",
                "updated_at": "2022-06-17T01:46:01.596472+00:00",
                "schedule": null,
                "tags": null,
                "parameters": [],
                "__typename": "queries",
                "visualizations": [
                    {
                        "id": 1604231,
                        "type": "table",
                        "name": "Query results",
                        "options": {},
                        "created_at": "2022-06-17T01:46:01.596472+00:00",
                        "__typename": "visualizations"
                    }
                ],
                "forked_query": null,
                "user": {
                    "id": 121830,
                    "name": "matrix2016",
                    "profile_image_url": null,
                    "__typename": "users"
                },
                "team": null,
                "query_favorite_count_all": null,
                "favorite_queries": []
            }
        }
    }
    "#;

    let res = serde_json::from_str::<UpsertQueryResponse>(json).unwrap();
    assert_eq!(res.query_id(), 917003);
}
