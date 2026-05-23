mod ai;
mod db;

use ai::{EvaluateRequest, HintRequest, LlmClient, ModelsRequest};
use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

struct AppState {
    llm_client: LlmClient,
    db_pool: sqlx::PgPool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct Deck {
    id: uuid::Uuid,
    name: String,
    description: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct Card {
    id: uuid::Uuid,
    deck_id: uuid::Uuid,
    vocab: String,
    reading: Option<String>,
    french_translation: String,
    anime_reference: Option<String>,
    context_sentence: Option<String>,
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
        "postgres://postgres:postgrespassword@localhost:5432/learnwithmanga".to_string()
    });

    // Initialisation de la base de données
    let db_pool = match db::init_db(&database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("⚠️ Impossible de démarrer la base de données : {}", e);
            eprintln!("⚠️ Le serveur fonctionnera SANS base de données (seuls les appels IA directs fonctionneront).");
            // Crée un pool temporaire ou panique ? On préfère paniquer ou forcer l'arrêt pour que l'utilisateur ajuste sa conf.
            panic!("Arrêt du serveur en raison de l'absence de base de données.");
        }
    };

    let llm_client = LlmClient::new();
    let shared_state = Arc::new(AppState {
        llm_client,
        db_pool,
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
        // Routes Base de Données
        .route("/api/db/seed", post(handle_seed_db))
        .route("/api/decks", get(handle_get_decks))
        .route("/api/cards", get(handle_get_all_cards))
        .route("/api/decks/:id/cards", get(handle_get_deck_cards))
        .route("/api/user/llm-settings/:user_id", get(handle_get_user_settings))
        .route("/api/user/llm-settings/:user_id/all", get(handle_get_all_user_settings))
        .route("/api/user/llm-settings", post(handle_save_user_settings))
        .route("/api/user/llm-settings/activate", post(handle_activate_user_setting))
        .route("/api/user/llm-settings/:user_id/:config_name", delete(handle_delete_user_setting))
        // Servir les fichiers statiques du frontend Vue
        .nest_service(
            "/",
            tower_http::services::ServeDir::new("../frontend/dist")
                .fallback(tower_http::services::ServeFile::new("../frontend/dist/index.html")),
        )
        .layer(cors)
        .with_state(shared_state);

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 Serveur backend LearnWithManga démarré sur http://{}", addr);

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
    println!("📥 Évaluation requise pour: \"{}\" | Réponse: \"{}\" | Modèle: {:?}", payload.vocab, payload.user_answer, payload.model);
    
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
    println!("📥 Indice requis pour: \"{}\" | Tier: {} | Modèle: {:?}", payload.vocab, payload.tier, payload.model);
    
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
    println!("📥 Récupération des paramètres LLM actifs pour l'utilisateur: {}", user_id);
    
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
                Err((StatusCode::NOT_FOUND, "Aucune configuration LLM existante".to_string()))
            }
        }
    }
}

async fn handle_get_all_user_settings(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<UserLlmSettings>>, (StatusCode, String)> {
    println!("📥 Récupération de toutes les configurations LLM pour l'utilisateur: {}", user_id);
    
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
    println!("📥 Enregistrement des paramètres LLM pour l'utilisateur: {} | Config: {}", payload.user_id, payload.config_name);
    
    let mut tx = state.db_pool.begin().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Transaction error: {}", e)))?;
        
    sqlx::query(
        "UPDATE user_llm_settings SET is_active = false WHERE user_id = $1"
    )
    .bind(payload.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur de désactivation des configurations: {}", e)))?;

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

    tx.commit().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur commit transaction: {}", e)))?;

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
    println!("📥 Activation de la configuration LLM: {} pour l'utilisateur: {}", payload.config_name, payload.user_id);
    
    let mut tx = state.db_pool.begin().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Transaction error: {}", e)))?;
        
    sqlx::query(
        "UPDATE user_llm_settings SET is_active = false WHERE user_id = $1"
    )
    .bind(payload.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur de désactivation globale: {}", e)))?;

    let rows_affected = sqlx::query(
        "UPDATE user_llm_settings SET is_active = true WHERE user_id = $1 AND config_name = $2"
    )
    .bind(payload.user_id)
    .bind(&payload.config_name)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur d'activation du profil: {}", e)))?
    .rows_affected();

    if rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, "Configuration introuvable".to_string()));
    }

    tx.commit().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur commit: {}", e)))?;

    Ok(Json(serde_json::json!({ "status": "success", "activated": payload.config_name })))
}

async fn handle_delete_user_setting(
    State(state): State<Arc<AppState>>,
    Path((user_id, config_name)): Path<(uuid::Uuid, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    println!("📥 Suppression de la configuration LLM: {} pour l'utilisateur: {}", config_name, user_id);
    
    sqlx::query(
        "DELETE FROM user_llm_settings WHERE user_id = $1 AND config_name = $2"
    )
    .bind(user_id)
    .bind(&config_name)
    .execute(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur BDD lors de la suppression: {}", e)))?;

    let remaining_active_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_llm_settings WHERE user_id = $1 AND is_active = true"
    )
    .bind(user_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur BDD comptage configurations actives: {}", e)))?;

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

    Ok(Json(serde_json::json!({ "status": "success", "deleted": config_name })))
}

// ==========================================
// Handlers Base de Données
// ==========================================

/// Seeder la base de données avec nos exemples favoris
async fn handle_seed_db(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    println!("📥 Requête de seeding de la base de données");

    // Vérifie s'il y a déjà des paquets en base
    let deck_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM decks")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur comptage decks: {}", e)))?;

    if deck_count > 0 {
        println!("ℹ️ La base de données contient déjà des decks. Seeding annulé.");
        return Ok(Json(serde_json::json!({
            "status": "success",
            "message": "La base de données est déjà initialisée."
        })));
    }

    // Crée le premier deck
    let deck_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO decks (id, name, description) VALUES ($1, $2, $3)"
    )
    .bind(deck_id)
    .bind("Animés Légendaires")
    .bind("Un paquet contenant les expressions cultes et mots clés issus de vos mangas et animés préférés.")
    .execute(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Échec création deck: {}", e)))?;

    // Insère nos 4 mots phares
    let seed_cards = vec![
        (
            "諦める",
            "あきらめる",
            "renoncer / abandonner",
            "Général / Commun",
            "Ce verbe est couramment utilisé dans les animés lors des moments dramatiques, souvent sous la forme de négation (諦めるな - Akirameruna - N'abandonne pas !)"
        ),
        (
            "お前はもう死んでいる",
            "おまえはもうしんでいる",
            "tu es déjà mort",
            "Hokuto no Ken",
            "La réplique mythique de Kenshiro juste avant l'explosion de son adversaire."
        ),
        (
            "心臓を捧げよ",
            "しんぞうをささげよ",
            "offrez vos cœurs / dédiez vos cœurs",
            "Shingeki no Kyojin",
            "Le cri de ralliement emblématique du Bataillon d'exploration mené par Erwin Smith."
        ),
        (
            "螺旋丸",
            "らせんがん",
            "l'orbe tourbillonnant",
            "Naruto",
            "Une technique ninja surpuissante créée par Minato Namikaze et perfectionnée par Naruto Uzumaki."
        )
    ];

    let mut inserted_count = 0;
    for (vocab, reading, french, anime, context) in seed_cards {
        sqlx::query(
            "INSERT INTO cards (id, deck_id, vocab, reading, french_translation, anime_reference, context_sentence) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(uuid::Uuid::new_v4())
        .bind(deck_id)
        .bind(vocab)
        .bind(reading)
        .bind(french)
        .bind(anime)
        .bind(context)
        .execute(&state.db_pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Échec insertion carte {}: {}", vocab, e)))?;
        
        inserted_count += 1;
    }

    println!("✅ Seeding complété ! {} cartes insérées.", inserted_count);
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Base de données initialisée avec {} cartes dans le deck 'Animés Légendaires'.", inserted_count)
    })))
}

/// Récupérer tous les Decks
async fn handle_get_decks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Deck>>, (StatusCode, String)> {
    println!("📥 Récupération des paquets de cartes");
    
    let decks = sqlx::query_as::<_, Deck>("SELECT id, name, description FROM decks ORDER BY name")
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Impossible de charger les decks: {}", e)))?;

    Ok(Json(decks))
}

/// Récupérer toutes les cartes
async fn handle_get_all_cards(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Card>>, (StatusCode, String)> {
    println!("📥 Récupération de l'intégralité des cartes");
    
    let cards = sqlx::query_as::<_, Card>(
        "SELECT id, deck_id, vocab, reading, french_translation, anime_reference, context_sentence FROM cards"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Impossible de charger les cartes: {}", e)))?;

    Ok(Json(cards))
}

/// Récupérer les cartes d'un deck précis
async fn handle_get_deck_cards(
    State(state): State<Arc<AppState>>,
    Path(deck_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<Card>>, (StatusCode, String)> {
    println!("📥 Récupération des cartes pour le deck: {}", deck_id);
    
    let cards = sqlx::query_as::<_, Card>(
        "SELECT id, deck_id, vocab, reading, french_translation, anime_reference, context_sentence \
         FROM cards WHERE deck_id = $1"
    )
    .bind(deck_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Impossible de charger les cartes du deck: {}", e)))?;

    Ok(Json(cards))
}
