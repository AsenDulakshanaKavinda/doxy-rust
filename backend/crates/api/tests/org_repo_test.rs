// ============================================================================
// SETUP — read this before running the tests below
// ============================================================================
//
// ### - Unit test for the Organization Repo
//
// These tests call your axum handlers directly (no HTTP server involved) but
// they go through a real Postgres connection via Diesel, so you need an
// actual test database. There's no practical way to "mock" Diesel's query
// builder for a repository layer this thin — you'd just be testing the mock.
//
// 1. Spin up a disposable Postgres instance:
//
//      docker run --name doxy-test-db \
//        -e POSTGRES_USER=postgres \
//        -e POSTGRES_PASSWORD=postgres \
//        -e POSTGRES_DB=doxy_test \
//        -p 5433:5432 \
//        -d postgres:17
//
//    (Using port 5433 so it doesn't collide with a dev Postgres on 5432.)
//
// 2. Point Diesel's CLI at it and run your migrations:
//
//      export TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5433/doxy_test
//      diesel migration run --database-url $TEST_DATABASE_URL
//
// 3. Export TEST_DATABASE_URL in your shell (or add it to a `.env.test` and
//    load it in the test helper below) whenever you run `cargo test`.
//
// 4. Add dev-dependencies to crates/api/Cargo.toml:
//
//      [dev-dependencies]
//      tokio = { version = "1", features = ["full", "test-util"] }
//      http-body-util = "0.1"
//      serde_json = "1"
//
// 5. Isolation strategy: each test builds its own pool with max_size(1) and
//    immediately calls `begin_test_transaction()` on that single connection.
//    Because the pool never grows past one connection, every `pool.get()`
//    call your handlers make later in that test (including from inside
//    `spawn_blocking`) hands back that same connection — still inside the
//    open transaction. Diesel/r2d2 never commits or resets it, so when the
//    pool is dropped at the end of the test, everything you wrote vanishes.
//    No manual cleanup, no test ordering issues, no `serial_test` needed —
//    every test gets a fully isolated view of the schema.
//
// Adjust `crate::db::pool` / `crate::db::repositories::user_repo` import
// paths below if your module layout differs.
// ============================================================================

#[cfg(test)]
mod tests {
    use api::db::pool::DbPool;
    use api::db::repositories::org_repo::{
        CreateOrganizationRequest, UpdateOrganizationRequest, create_organization,
        delete_organization_by_id, get_organization_by_id, update_organization_by_id,
    };
    use axum::Json;
    use axum::extract::{Path, State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use diesel::r2d2::{ConnectionManager, Pool};
    use diesel::{Connection, PgConnection};
    use http_body_util::BodyExt;
    use serde_json::Value;

    /// Builds a single-connection pool with an open, never-committed
    /// transaction. See the setup notes above for why max_size(1) matters.
    fn test_pool() -> DbPool {
        dotenvy::dotenv().ok();

        let db_url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set to run these tests");

        let manager = ConnectionManager::<PgConnection>::new(db_url);
        let pool = Pool::builder()
            .max_size(1)
            .build(manager)
            .expect("failed to build test pool");

        {
            let mut conn = pool.get().expect("failed to check out test connection");
            conn.begin_test_transaction()
                .expect("failed to begin test transaction");
        }

        pool
    }

    /// Pulls the status code and parsed JSON body out of an IntoResponse
    /// value, since the handlers return `impl IntoResponse` and hide the
    /// concrete Ok type from callers.
    async fn read_response(response: impl IntoResponse) -> (StatusCode, Value) {
        let response = response.into_response();
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("failed to read body")
            .to_bytes();

        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).expect("response body was not valid JSON")
        };

        (status, body)
    }

    /// Stand-in for `Result::expect_err` — the handlers return
    /// `impl IntoResponse` as their Ok type, which doesn't implement
    /// `Debug`, so the standard library's `expect_err`/`unwrap_err`
    /// (both bounded on `T: Debug`) won't compile here.
    fn expect_err<T>(result: Result<T, (StatusCode, String)>, msg: &str) -> (StatusCode, String) {
        // Deliberately avoids matching on bare `Ok`/`Err` patterns: something
        // elsewhere in this crate's dependency tree (a gRPC/telemetry crate,
        // per the compiler's `grpc_errors_as_failures.rs` note) glob-imports
        // its own unit variant named `Ok`, which shadows `Result::Ok` at
        // pattern-match sites. `Result::err()` sidesteps it entirely.
        result.err().unwrap_or_else(|| panic!("{msg}"))
    }

    fn valid_create_request(name: &str) -> CreateOrganizationRequest {
        CreateOrganizationRequest {
            name: name.to_string(),
            slug: "test-slug".to_string(),
            logo: None,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn create_organization_succeeds_with_valid_payload() {
        let pool = test_pool();

        let response =
            create_organization(State(pool), Json(valid_create_request("Peter parker"))).await;
        let (status, body) = read_response(response).await;

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["name"], "Peter parker");
        assert_eq!(body["slug"], "test-slug");
        assert!(body["id"].is_string(), "expected a generated id");
    }

    #[tokio::test]
    async fn create_organization_rejects_empty_name() {
        let pool = test_pool();

        let payload = CreateOrganizationRequest {
            name: "   ".to_string(),
            slug: "test-slug".to_string(),
            logo: None,
            metadata: None,
        };

        let err = expect_err(
            create_organization(State(pool), Json(payload)).await,
            "expected Err for empty name",
        );

        assert_eq!(err.0, StatusCode::BAD_REQUEST)
    }

    #[tokio::test]
    async fn create_organization_rejects_invalid_slug() {
        let pool = test_pool();

        let payload = CreateOrganizationRequest {
            name: "test_org".to_string(),
            slug: "Invalid Slug!".to_string(),
            logo: None,
            metadata: None,
        };

        let err = expect_err(
            create_organization(State(pool), Json(payload)).await,
            "expected Err for invalid slug",
        );

        assert_eq!(err.0, StatusCode::BAD_REQUEST)
    }

    #[tokio::test]
    async fn create_organization_rejects_empty_slug() {
        let pool = test_pool();

        let payload = CreateOrganizationRequest {
            name: "test_orgs".to_string(),
            slug: " ".to_string(),
            logo: None,
            metadata: None,
        };

        let err = expect_err(
            create_organization(State(pool), Json(payload)).await,
            "expected Err for empty slug",
        );

        assert_eq!(err.0, StatusCode::BAD_REQUEST)
    }

    #[tokio::test]
    async fn get_organization_by_id_returns_created_organization() {
        let pool = test_pool();

        let created = create_organization(
            State(pool.clone()),
            Json(valid_create_request("Peter parker")),
        )
        .await
        .expect("create_organization should succeed");

        let (_, created_body) = read_response(created).await;
        let id = created_body["id"].as_str().unwrap().to_string();

        let fetched = get_organization_by_id(State(pool), Path(id.clone()))
            .await
            .expect("get_user_by_id should succeed");
        let (status, body) = read_response(fetched).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["id"], id);
        assert_eq!(body["name"], "Peter parker");
    }

    #[tokio::test]
    async fn get_organization_by_id_returns_404_when_missing() {
        let pool = test_pool();
        let missing_id = uuid::Uuid::new_v4().to_string();

        let err = expect_err(
            get_organization_by_id(State(pool), Path(missing_id)).await,
            "expected Err for missing user",
        );

        assert_eq!(err.0, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn update_organization_by_id_applies_changes() {
        let pool = test_pool();

        let created = create_organization(
            State(pool.clone()),
            Json(valid_create_request("org-name-before")),
        )
        .await
        .expect("create_organization should be succeed");

        let (_, created_body) = read_response(created).await;
        let id = created_body["id"].as_str().unwrap().to_string();

        let update_payload = UpdateOrganizationRequest {
            name: Some("org-name-after".to_string()),
            slug: None,
            logo: None,
            metadata: None,
        };

        let updated =
            update_organization_by_id(State(pool), Path(id.clone()), Json(update_payload))
                .await
                .expect("update_organization_by_id should succeed");
        let (status, body) = read_response(updated).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["name"], "org-name-after");
    }

    #[tokio::test]
    async fn update_organization_by_id_rejects_empty_name() {
        let pool = test_pool();

        let created = create_organization(
            State(pool.clone()),
            Json(valid_create_request("org-name-before")),
        )
        .await
        .expect("create_organization should be succeed");

        let (_, created_body) = read_response(created).await;
        let id = created_body["id"].as_str().unwrap().to_string();

        let update_payload = UpdateOrganizationRequest {
            name: Some("   ".to_string()),
            slug: None,
            logo: None,
            metadata: None,
        };

        let err = expect_err(
            update_organization_by_id(State(pool), Path(id), Json(update_payload)).await,
            "expected Err for invalid slug body",
        );

        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_organization_by_id_rejects_invalid_slug() {
        let pool = test_pool();

        let created = create_organization(
            State(pool.clone()),
            Json(valid_create_request("org-name-before")),
        )
        .await
        .expect("create_organization should be succeed");

        let (_, created_body) = read_response(created).await;
        let id = created_body["id"].as_str().unwrap().to_string();

        let update_payload = UpdateOrganizationRequest {
            name: None,
            slug: Some("Invalid Slug!".to_string()),
            logo: None,
            metadata: None,
        };

        let err = expect_err(
            update_organization_by_id(State(pool), Path(id), Json(update_payload)).await,
            "expected Err for invalid slug body",
        );

        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delete_organization_by_id_removes_organization() {
        let pool = test_pool();

        let created = create_organization(
            State(pool.clone()),
            Json(valid_create_request("org-name-before")),
        )
        .await
        .expect("create_organization should be succeed");
        let (_, created_body) = read_response(created).await;
        let id = created_body["id"].as_str().unwrap().to_string();

        let deleted = delete_organization_by_id(State(pool.clone()), Path(id.clone()))
            .await
            .expect("delete_user_by_id should succeed");
        let (status, _) = read_response(deleted).await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        let err = expect_err(
            get_organization_by_id(State(pool), Path(id)).await,
            "expected Err after deletion",
        );
        assert_eq!(err.0, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_user_by_id_returns_404_when_missing() {
        let pool = test_pool();
        let missing_id = uuid::Uuid::new_v4().to_string();

        let err = expect_err(
            delete_organization_by_id(State(pool), Path(missing_id)).await,
            "expected Err for missing user",
        );

        assert_eq!(err.0, StatusCode::NOT_FOUND);
    }
}
