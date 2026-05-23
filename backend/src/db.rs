use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

/// Initialise le pool de connexions SQLx pour PostgreSQL
/// et exécute automatiquement les migrations SQL intégrées.
pub async fn init_db(database_url: &str) -> Result<PgPool, String> {
    println!("🔌 Connexion à la base de données PostgreSQL/Supabase...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await
        .map_err(|e| format!("Échec de la connexion à la base de données: {}", e))?;

    println!("✅ Connexion établie avec succès !");

    // Exécute automatiquement les migrations intégrées lors du démarrage
    println!("⚙️ Exécution des migrations de base de données...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Échec de l'exécution des migrations: {}", e))?;

    println!("✅ Base de données migrée et prête à l'emploi.");

    Ok(pool)
}
