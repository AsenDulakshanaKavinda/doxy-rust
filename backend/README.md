# Backend Workspace Architecture

## Crate Responsibilities & Boundaries

* **`shared`** (`crates/shared`)
  * **Role:** Houses common data structures, primitive types, shared errors, and stateless utilities.
  * **Dependencies:** None (zero internal workspace dependencies).

* **`core_domain`** (`crates/core`)
  * **Role:** Encapsulates business logic, domain entities, and core application rules.
  * **Dependencies:** Depends strictly on `shared`. Decoupled from transport and framework logic.

* **`api`** (`crates/api`)
  * **Role:** Application entry point handling routing, HTTP handlers, and external middleware.
  * **Dependencies:** Depends on both `core_domain` and `shared`.

## Build Verification

Build all crates simultaneously from the root:

```bash
cargo build --workspace
```

### structure
```
doxy-rust/
├── docker-compose.yaml
├── LICENSE
├── README.md
└── backend/
    ├── Cargo.toml                 # workspace
    ├── diesel.toml
    ├── migrations/                # Diesel migrations (workspace root, one source of truth)
    └── crates/
        ├── shared/
        │   ├── Cargo.toml
        │   └── src/
        │       ├── lib.rs
        │       ├── error.rs        # AppError, shared Result alias
        │       ├── ids.rs          # newtype IDs: OrgId, UserId, DocumentId, ChunkId...
        │       └── pagination.rs
        │
        ├── core/
        │   ├── Cargo.toml
        │   └── src/
        │       ├── lib.rs
        │       ├── domain/         # pure entities + invariants, no I/O
        │       │   ├── mod.rs
        │       │   ├── org.rs
        │       │   ├── user.rs
        │       │   ├── membership.rs
        │       │   ├── document.rs
        │       │   ├── chunk.rs
        │       │   ├── subscription.rs
        │       │   └── usage.rs
        │       ├── ports/          # traits `api` must implement (dependency inversion)
        │       │   ├── mod.rs
        │       │   ├── repository.rs   # OrgRepo, UserRepo, DocumentRepo, SubscriptionRepo
        │       │   ├── vector_store.rs # upsert/search/delete
        │       │   ├── embedder.rs
        │       │   ├── llm.rs          # chat/stream trait
        │       │   ├── storage.rs      # raw file storage
        │       │   └── payment.rs
        │       └── services/       # use-case orchestration — depends only on ports+domain
        │           ├── mod.rs
        │           ├── tenancy_service.rs
        │           ├── ingestion_service.rs
        │           ├── rag_service.rs
        │           ├── billing_service.rs
        │           └── admin_service.rs
        │
        └── api/
            ├── Cargo.toml
            └── src/
                ├── main.rs
                ├── config.rs
                ├── state.rs         # AppState: pools, clients, injected into services
                ├── middleware/
                │   ├── mod.rs
                │   ├── auth.rs      # session/JWT validation
                │   ├── tenant.rs    # resolves org from request, scopes queries
                │   └── rbac.rs
                ├── routes/
                │   ├── mod.rs
                │   ├── health.rs
                │   ├── auth.rs
                │   ├── orgs.rs
                │   ├── documents.rs
                │   ├── chat.rs
                │   ├── billing.rs
                │   └── admin.rs
                ├── auth/            # OAuth-specific
                │   ├── mod.rs
                │   ├── google.rs
                │   ├── github.rs
                │   └── session.rs
                ├── db/              # Diesel adapter — implements core::ports::repository
                │   ├── mod.rs
                │   ├── schema.rs    # diesel-generated, do not hand-edit
                │   ├── models.rs
                │   ├── pool.rs
                │   └── repositories/
                │       ├── org_repo.rs
                │       ├── user_repo.rs
                │       ├── document_repo.rs
                │       └── subscription_repo.rs
                ├── vector/          # Qdrant adapter — implements core::ports::vector_store
                │   ├── mod.rs
                │   └── qdrant_store.rs
                ├── llm/             # rig-core adapter — implements core::ports::{llm,embedder}
                │   ├── mod.rs
                │   ├── rig_agent.rs
                │   └── embedder.rs
                ├── storage/         # local disk / S3 adapter
                │   ├── mod.rs
                │   ├── local.rs
                │   └── s3.rs
                ├── billing/         # Stripe adapter
                │   ├── mod.rs
                │   ├── stripe_client.rs
                │   └── webhook.rs
                └── jobs/            # background ingestion worker
                    ├── mod.rs
                    ├── worker.rs
                    └── ingestion_job.rs
```