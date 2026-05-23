mod ai;
mod catalog;
mod db;

use ai::{EvaluateRequest, HintRequest, LlmClient, ModelsRequest};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{Method, StatusCode},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

struct AppState {
    llm_client: LlmClient,
    db_pool: sqlx::PgPool,
    catalog: catalog::CardCatalog,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserRole {
    user_id: Option<uuid::Uuid>,
    email: String,
    role: String,
    granted_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct AddAdminRequest {
    email: String,
}

#[derive(Debug, Deserialize)]
struct ClaimAdminRequest {
    user_id: uuid::Uuid,
    email: String,
}

#[derive(Debug, Deserialize)]
struct LinkAdminRequest {
    user_id: uuid::Uuid,
    email: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
struct UserLlmSettings {
    user_id: uuid::Uuid,
    config_name: String,
    api_url: String,
    api_key: Option<String>,
    model: String,
    temperature_eval: f32,
    temperature_hint: f32,
    pub top_p: f32,
    pub frequency_penalty: f32,
    pub max_tokens: i32,
    pub is_active: bool,
}

#[tokio::main]
async fn main() {
    // Charge les variables d'environnement depuis le fichier .env si présent
    let _ = dotenvy::dotenv();

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgrespassword@localhost:5432/learnwithanime".to_string()
    });

    // Initialisation de la base de données
    let db_pool = match db::init_db(&database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("⚠️ Impossible de démarrer la base de données : {}", e);
            eprintln!(
                "⚠️ Le serveur fonctionnera SANS base de données (seuls les appels IA directs fonctionneront)."
            );
            // Crée un pool temporaire ou panique ? On préfère paniquer ou forcer l'arrêt pour que l'utilisateur ajuste sa conf.
            panic!("Arrêt du serveur en raison de l'absence de base de données.");
        }
    };

    // Seed default admin on first startup
    let admin_email = "devops.benjamin@gmail.com";
    let result = sqlx::query(
        "INSERT INTO user_roles (email, role) VALUES ($1, 'admin') ON CONFLICT (email) DO NOTHING",
    )
    .bind(admin_email)
    .execute(&db_pool)
    .await;
    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                println!("👑 Admin par défaut créé: {}", admin_email);
            }
        }
        Err(e) => eprintln!("⚠️ Impossible de créer l'admin par défaut: {}", e),
    }

    let llm_client = LlmClient::new();

    let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "data".to_string());
    let catalog = catalog::load_catalog(&PathBuf::from(&data_dir)).unwrap_or_else(|e| {
        panic!("❌ Erreur chargement catalogue: {}", e);
    });
    println!(
        "📚 Catalogue chargé: {} cartes, {} decks, {} kanji, {} radicaux",
        catalog.cards.len(),
        catalog.decks.len(),
        catalog.kanji.len(),
        catalog.radicals.len()
    );

    let shared_state = Arc::new(AppState {
        llm_client,
        db_pool,
        catalog,
    });

    // Configuration des CORS pour autoriser le frontend (port 5173 ou autre)
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any) // En local, autorise tout
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::DELETE])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        // Routes IA
        .route("/api/ai/evaluate", post(handle_evaluate))
        .route("/api/ai/hint", post(handle_hint))
        .route("/api/ai/models", post(handle_get_models))
        // Routes Catalogue (JSON)
        .route("/api/decks", get(handle_get_decks))
        .route("/api/decks/:slug", get(handle_get_deck_by_slug))
        .route("/api/kanji", get(handle_list_kanji))
        .route("/api/kanji/:kanji_char", get(handle_get_kanji))
        .route("/api/radicals", get(handle_list_radicals))
        .route("/api/radicals/:number", get(handle_get_radical))
        .route(
            "/api/user/llm-settings/:user_id",
            get(handle_get_user_settings),
        )
        .route(
            "/api/user/llm-settings/:user_id/all",
            get(handle_get_all_user_settings),
        )
        .route("/api/user/llm-settings", post(handle_save_user_settings))
        .route(
            "/api/user/llm-settings/activate",
            post(handle_activate_user_setting),
        )
        .route(
            "/api/user/llm-settings/:user_id/:config_name",
            delete(handle_delete_user_setting),
        )
        // Routes Admin
        .route("/api/admin/admins", get(handle_list_admins))
        .route("/api/admin/admins", post(handle_add_admin))
        .route("/api/admin/admins/:email", delete(handle_remove_admin))
        .route("/api/admin/claim", post(handle_claim_admin))
        .route("/api/admin/link", post(handle_link_admin))
        // Servir les fichiers statiques du frontend Vue
        .nest_service(
            "/",
            tower_http::services::ServeDir::new("../frontend/dist").fallback(
                tower_http::services::ServeFile::new("../frontend/dist/index.html"),
            ),
        )
        .layer(cors)
        .with_state(shared_state);

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!(
        "🚀 Serveur backend LearnWithAnime démarré sur http://{}",
        addr
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ==========================================
// Handlers IA
// ==========================================

async fn handle_evaluate(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<EvaluateRequest>,
) -> Result<Json<ai::EvaluateResponse>, (StatusCode, String)> {
    println!(
        "📥 Évaluation requise pour: \"{}\" | Réponse: \"{}\" | Modèle: {:?}",
        payload.vocab, payload.user_answer, payload.model
    );

    match state.llm_client.evaluate_answer(&payload).await {
        Ok(eval) => {
            println!("✅ Évaluation réussie: score={}", eval.score);
            Ok(Json(eval))
        }
        Err(err) => {
            eprintln!("❌ Erreur d'évaluation LLM: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, err))
        }
    }
}

async fn handle_hint(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<HintRequest>,
) -> Result<Json<ai::HintResponse>, (StatusCode, String)> {
    println!(
        "📥 Indice requis pour: \"{}\" | Tier: {} | Modèle: {:?}",
        payload.vocab, payload.tier, payload.model
    );

    match state.llm_client.generate_hint(&payload).await {
        Ok(hint) => {
            println!("💡 Indice généré avec succès");
            Ok(Json(hint))
        }
        Err(err) => {
            eprintln!("❌ Erreur de génération d'indice LLM: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, err))
        }
    }
}

async fn handle_get_models(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ModelsRequest>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    println!("📥 Requête pour récupérer la liste des modèles (dynamique)");

    match state.llm_client.get_models(&payload).await {
        Ok(models) => {
            println!("✅ Récupération réussie de {} modèles", models.len());
            Ok(Json(models))
        }
        Err(err) => {
            eprintln!("❌ Erreur de récupération des modèles LLM: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, err))
        }
    }
}

async fn handle_get_user_settings(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<uuid::Uuid>,
) -> Result<Json<UserLlmSettings>, (StatusCode, String)> {
    println!(
        "📥 Récupération des paramètres LLM actifs pour l'utilisateur: {}",
        user_id
    );

    let settings = sqlx::query_as::<_, UserLlmSettings>(
        "SELECT user_id, config_name, api_url, api_key, model, temperature_eval, temperature_hint, top_p, frequency_penalty, max_tokens, is_active \
         FROM user_llm_settings WHERE user_id = $1 AND is_active = true"
    )
    .bind(user_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur BDD lors du chargement des paramètres: {}", e)))?;

    match settings {
        Some(s) => Ok(Json(s)),
        None => {
            // Si aucune n'est active, on cherche la toute première disponible
            let fallback_settings = sqlx::query_as::<_, UserLlmSettings>(
                "SELECT user_id, config_name, api_url, api_key, model, temperature_eval, temperature_hint, top_p, frequency_penalty, max_tokens, is_active \
                 FROM user_llm_settings WHERE user_id = $1 ORDER BY updated_at DESC LIMIT 1"
            )
            .bind(user_id)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur BDD fallback: {}", e)))?;

            if let Some(mut fs) = fallback_settings {
                let _ = sqlx::query("UPDATE user_llm_settings SET is_active = true WHERE user_id = $1 AND config_name = $2")
                    .bind(user_id)
                    .bind(&fs.config_name)
                    .execute(&state.db_pool)
                    .await;
                fs.is_active = true;
                Ok(Json(fs))
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    "Aucune configuration LLM existante".to_string(),
                ))
            }
        }
    }
}

async fn handle_get_all_user_settings(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<UserLlmSettings>>, (StatusCode, String)> {
    println!(
        "📥 Récupération de toutes les configurations LLM pour l'utilisateur: {}",
        user_id
    );

    let configs = sqlx::query_as::<_, UserLlmSettings>(
        "SELECT user_id, config_name, api_url, api_key, model, temperature_eval, temperature_hint, top_p, frequency_penalty, max_tokens, is_active \
         FROM user_llm_settings WHERE user_id = $1 ORDER BY updated_at DESC"
    )
    .bind(user_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur BDD lors du chargement de la liste: {}", e)))?;

    Ok(Json(configs))
}

async fn handle_save_user_settings(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserLlmSettings>,
) -> Result<Json<UserLlmSettings>, (StatusCode, String)> {
    println!(
        "📥 Enregistrement des paramètres LLM pour l'utilisateur: {} | Config: {}",
        payload.user_id, payload.config_name
    );

    let mut tx = state.db_pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Transaction error: {}", e),
        )
    })?;

    sqlx::query("UPDATE user_llm_settings SET is_active = false WHERE user_id = $1")
        .bind(payload.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erreur de désactivation des configurations: {}", e),
            )
        })?;

    sqlx::query(
        "INSERT INTO user_llm_settings (user_id, config_name, api_url, api_key, model, temperature_eval, temperature_hint, top_p, frequency_penalty, max_tokens, is_active, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, CURRENT_TIMESTAMP) \
         ON CONFLICT (user_id, config_name) DO UPDATE SET \
         api_url = EXCLUDED.api_url, \
         api_key = EXCLUDED.api_key, \
         model = EXCLUDED.model, \
         temperature_eval = EXCLUDED.temperature_eval, \
         temperature_hint = EXCLUDED.temperature_hint, \
         top_p = EXCLUDED.top_p, \
         frequency_penalty = EXCLUDED.frequency_penalty, \
         max_tokens = EXCLUDED.max_tokens, \
         is_active = true, \
         updated_at = CURRENT_TIMESTAMP"
    )
    .bind(payload.user_id)
    .bind(&payload.config_name)
    .bind(&payload.api_url)
    .bind(&payload.api_key)
    .bind(&payload.model)
    .bind(payload.temperature_eval)
    .bind(payload.temperature_hint)
    .bind(payload.top_p)
    .bind(payload.frequency_penalty)
    .bind(payload.max_tokens)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Impossible d'enregistrer la configuration: {}", e)))?;

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur commit transaction: {}", e),
        )
    })?;

    let mut response_payload = payload.clone();
    response_payload.is_active = true;
    Ok(Json(response_payload))
}

#[derive(Debug, Deserialize)]
struct ActivateConfigRequest {
    user_id: uuid::Uuid,
    config_name: String,
}

async fn handle_activate_user_setting(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ActivateConfigRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    println!(
        "📥 Activation de la configuration LLM: {} pour l'utilisateur: {}",
        payload.config_name, payload.user_id
    );

    let mut tx = state.db_pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Transaction error: {}", e),
        )
    })?;

    sqlx::query("UPDATE user_llm_settings SET is_active = false WHERE user_id = $1")
        .bind(payload.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erreur de désactivation globale: {}", e),
            )
        })?;

    let rows_affected = sqlx::query(
        "UPDATE user_llm_settings SET is_active = true WHERE user_id = $1 AND config_name = $2",
    )
    .bind(payload.user_id)
    .bind(&payload.config_name)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur d'activation du profil: {}", e),
        )
    })?
    .rows_affected();

    if rows_affected == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "Configuration introuvable".to_string(),
        ));
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur commit: {}", e),
        )
    })?;

    Ok(Json(
        serde_json::json!({ "status": "success", "activated": payload.config_name }),
    ))
}

async fn handle_delete_user_setting(
    State(state): State<Arc<AppState>>,
    Path((user_id, config_name)): Path<(uuid::Uuid, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    println!(
        "📥 Suppression de la configuration LLM: {} pour l'utilisateur: {}",
        config_name, user_id
    );

    sqlx::query("DELETE FROM user_llm_settings WHERE user_id = $1 AND config_name = $2")
        .bind(user_id)
        .bind(&config_name)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erreur BDD lors de la suppression: {}", e),
            )
        })?;

    let remaining_active_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_llm_settings WHERE user_id = $1 AND is_active = true",
    )
    .bind(user_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur BDD comptage configurations actives: {}", e),
        )
    })?;

    if remaining_active_count == 0 {
        let newest_config: Option<String> = sqlx::query_scalar(
            "SELECT config_name FROM user_llm_settings WHERE user_id = $1 ORDER BY updated_at DESC LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur BDD sélection profil le plus récent: {}", e)))?;

        if let Some(c_name) = newest_config {
            sqlx::query(
                "UPDATE user_llm_settings SET is_active = true WHERE user_id = $1 AND config_name = $2"
            )
            .bind(user_id)
            .bind(&c_name)
            .execute(&state.db_pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur BDD activation du profil de secours: {}", e)))?;
        }
    }

    Ok(Json(
        serde_json::json!({ "status": "success", "deleted": config_name }),
    ))
}

// ==========================================
// Handlers Admin
// ==========================================

async fn handle_list_admins(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserRole>>, (StatusCode, String)> {
    println!("📥 Liste des administrateurs");

    let admins = sqlx::query_as::<_, (Option<uuid::Uuid>, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT user_id, email, role, granted_at FROM user_roles WHERE role = 'admin' ORDER BY granted_at"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur BDD: {}", e)))?
    .into_iter()
    .map(|(user_id, email, role, granted_at)| UserRole { user_id, email, role, granted_at })
    .collect();

    Ok(Json(admins))
}

async fn handle_add_admin(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddAdminRequest>,
) -> Result<Json<UserRole>, (StatusCode, String)> {
    println!("📥 Ajout d'un administrateur par email: {}", payload.email);

    let row = sqlx::query_as::<_, (Option<uuid::Uuid>, String, String, chrono::DateTime<chrono::Utc>)>(
        "INSERT INTO user_roles (email, role) VALUES ($1, 'admin') ON CONFLICT (email) DO UPDATE SET role = 'admin' RETURNING user_id, email, role, granted_at"
    )
    .bind(&payload.email)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur BDD: {}", e)))?;

    println!("✅ Administrateur ajouté: {}", payload.email);
    Ok(Json(UserRole {
        user_id: row.0,
        email: row.1,
        role: row.2,
        granted_at: row.3,
    }))
}

async fn handle_remove_admin(
    State(state): State<Arc<AppState>>,
    Path(email): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    println!("📥 Suppression d'un administrateur: {}", email);

    let rows = sqlx::query("DELETE FROM user_roles WHERE email = $1 AND role = 'admin'")
        .bind(&email)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erreur BDD: {}", e),
            )
        })?
        .rows_affected();

    if rows == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "Administrateur introuvable".to_string(),
        ));
    }

    println!("✅ Administrateur supprimé: {}", email);
    Ok(Json(
        serde_json::json!({ "status": "success", "removed": email }),
    ))
}

async fn handle_claim_admin(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ClaimAdminRequest>,
) -> Result<Json<UserRole>, (StatusCode, String)> {
    println!(
        "📥 Réclamation du rôle admin: {} ({})",
        payload.user_id, payload.email
    );

    let admin_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM user_roles WHERE role = 'admin'")
            .fetch_one(&state.db_pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Erreur BDD: {}", e),
                )
            })?;

    if admin_count > 0 {
        return Err((StatusCode::FORBIDDEN, "Un administrateur existe déjà. Seul un admin existant peut ajouter de nouveaux admins.".to_string()));
    }

    let row = sqlx::query_as::<_, (Option<uuid::Uuid>, String, String, chrono::DateTime<chrono::Utc>)>(
        "INSERT INTO user_roles (user_id, email, role) VALUES ($1, $2, 'admin') RETURNING user_id, email, role, granted_at"
    )
    .bind(payload.user_id)
    .bind(&payload.email)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur BDD: {}", e)))?;

    println!(
        "✅ Premier administrateur créé: {} ({})",
        payload.user_id, payload.email
    );
    Ok(Json(UserRole {
        user_id: row.0,
        email: row.1,
        role: row.2,
        granted_at: row.3,
    }))
}

async fn handle_link_admin(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LinkAdminRequest>,
) -> Result<Json<UserRole>, (StatusCode, String)> {
    println!(
        "📥 Liaison du compte admin: {} -> {}",
        payload.email, payload.user_id
    );

    let row = sqlx::query_as::<_, (Option<uuid::Uuid>, String, String, chrono::DateTime<chrono::Utc>)>(
        "UPDATE user_roles SET user_id = $1 WHERE email = $2 AND role = 'admin' AND user_id IS NULL RETURNING user_id, email, role, granted_at"
    )
    .bind(payload.user_id)
    .bind(&payload.email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur BDD: {}", e)))?;

    match row {
        Some(r) => {
            println!("✅ Compte lié: {} -> {}", payload.email, payload.user_id);
            Ok(Json(UserRole {
                user_id: r.0,
                email: r.1,
                role: r.2,
                granted_at: r.3,
            }))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            "Aucune invitation admin trouvée pour cet email ou déjà liée".to_string(),
        )),
    }
}

// ==========================================
// Handlers Catalogue (JSON)
// ==========================================

/// Récupérer tous les Decks (métadonnées seulement)
async fn handle_get_decks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DeckMeta>>, (StatusCode, String)> {
    let mut decks: Vec<DeckMeta> = state
        .catalog
        .decks
        .values()
        .map(|d| DeckMeta {
            slug: d.slug.clone(),
            name: d.name.clone(),
            description: d.description.clone(),
            card_count: d.cards.len(),
        })
        .collect();
    decks.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(decks))
}

/// Récupérer un deck avec ses cartes hydratées
async fn handle_get_deck_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<catalog::HydratedDeck>, (StatusCode, String)> {
    let deck = state.catalog.decks.get(&slug).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Deck '{}' introuvable", slug),
        )
    })?;

    let hydrated = catalog::HydratedDeck::from_deck(deck, &state.catalog);
    Ok(Json(hydrated))
}

#[derive(Debug, Serialize)]
struct DeckMeta {
    slug: String,
    name: String,
    description: Option<String>,
    card_count: usize,
}

// ==========================================
// Handlers Kanji
// ==========================================

/// Lister tous les kanji (métadonnées légères : char + jlpt_level + radical_number)
async fn handle_list_kanji(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<KanjiMeta>>, (StatusCode, String)> {
    let mut list: Vec<KanjiMeta> = state
        .catalog
        .kanji
        .values()
        .map(|k| KanjiMeta {
            char: k.char.clone(),
            jlpt_level: k.jlpt_level,
            radical_number: k.radical_number,
            stroke_count: k.stroke_count,
            frequency_rank: k.frequency_rank,
        })
        .collect();
    list.sort_by_key(|k| k.frequency_rank.unwrap_or(u32::MAX));
    Ok(Json(list))
}

/// Récupérer un kanji complet par son caractère
async fn handle_get_kanji(
    State(state): State<Arc<AppState>>,
    Path(kanji_char): Path<String>,
) -> Result<Json<catalog::KanjiEntry>, (StatusCode, String)> {
    let k = state.catalog.kanji.get(&kanji_char).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Kanji '{}' introuvable", kanji_char),
        )
    })?;
    Ok(Json(k.clone()))
}

/// Lister tous les radicaux
async fn handle_list_radicals(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<catalog::Radical>>, (StatusCode, String)> {
    let mut list: Vec<catalog::Radical> = state.catalog.radicals.values().cloned().collect();
    list.sort_by_key(|r| r.number);
    Ok(Json(list))
}

/// Récupérer un radical par son numéro
async fn handle_get_radical(
    State(state): State<Arc<AppState>>,
    Path(number): Path<u8>,
) -> Result<Json<catalog::Radical>, (StatusCode, String)> {
    let r = state.catalog.radicals.get(&number).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Radical {} introuvable", number),
        )
    })?;
    Ok(Json(r.clone()))
}

#[derive(Debug, Serialize)]
struct KanjiMeta {
    char: String,
    jlpt_level: u8,
    radical_number: Option<u8>,
    stroke_count: Option<u8>,
    frequency_rank: Option<u32>,
}
