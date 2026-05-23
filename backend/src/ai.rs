use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoiceMessage {
    pub content: String,
}

// Structures pour l'évaluation
#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluateRequest {
    pub vocab: String,
    pub user_answer: String,
    pub context: Option<String>,
    pub model: Option<String>,    // Modèle dynamique choisi par le frontend
    pub temperature: Option<f32>, // Température dynamique choisie par le frontend
    pub api_url: Option<String>,  // URL de l'API LLM dynamique choisie par le frontend
    pub api_key: Option<String>, // Clé d'API LLM dynamique (ex: OpenAI key) fournie par l'utilisateur
    pub top_p: Option<f32>,
    pub max_tokens: Option<u32>,
    pub frequency_penalty: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluateResponse {
    pub is_correct: bool,
    pub score: u8,
    pub explanation: String,
    pub correction: Option<String>,
}

// Structures pour les indices
#[derive(Debug, Serialize, Deserialize)]
pub struct HintRequest {
    pub vocab: String,
    pub tier: u8,                 // 1 ou 2
    pub model: Option<String>,    // Modèle dynamique choisi par le frontend
    pub temperature: Option<f32>, // Température dynamique choisie par le frontend
    pub api_url: Option<String>,  // URL de l'API LLM dynamique
    pub api_key: Option<String>,  // Clé d'API LLM dynamique
    pub top_p: Option<f32>,
    pub max_tokens: Option<u32>,
    pub frequency_penalty: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HintResponse {
    pub hint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsRequest {
    pub api_url: Option<String>,
    pub api_key: Option<String>,
}

// Structures pour récupérer la liste des modèles
#[derive(Debug, Deserialize)]
pub struct ModelListResponse {
    pub data: Vec<ModelItem>,
}

#[derive(Debug, Deserialize)]
pub struct ModelItem {
    pub id: String,
}

pub struct LlmClient {
    client: reqwest::Client,
    api_url: String,
    model: String,
}

impl LlmClient {
    pub fn new() -> Self {
        let api_url =
            env::var("LLM_API_URL").unwrap_or_else(|_| "http://localhost:1337/v1".to_string());
        let model = env::var("LLM_MODEL_NAME").unwrap_or_else(|_| "minimax-2.7b".to_string());

        Self {
            client: reqwest::Client::new(),
            api_url,
            model,
        }
    }

    /// Nettoie la réponse en texte pour extraire uniquement le JSON valide, au cas où le LLM
    /// ajoute des balises markdown de type ```json ou du texte superflu.
    fn clean_json_response(&self, text: &str) -> String {
        let mut cleaned = text.trim().to_string();

        // Supprime ```json au début si présent
        if cleaned.starts_with("```json") {
            cleaned = cleaned.trim_start_matches("```json").to_string();
        } else if cleaned.starts_with("```") {
            cleaned = cleaned.trim_start_matches("```").to_string();
        }

        // Supprime ``` à la fin si présent
        if cleaned.ends_with("```") {
            cleaned = cleaned.trim_end_matches("```").to_string();
        }

        cleaned.trim().to_string()
    }

    pub async fn evaluate_answer(&self, req: &EvaluateRequest) -> Result<EvaluateResponse, String> {
        let api_url_to_use = req.api_url.clone().unwrap_or_else(|| self.api_url.clone());
        let endpoint = format!("{}/chat/completions", api_url_to_use.trim_end_matches('/'));
        let system_prompt = "Tu es un tuteur de japonais bienveillant pour l'application LearnWithAnime. \
Évalue la réponse de l'utilisateur pour le mot ou l'expression japonaise. \
Réponds UNIQUEMENT sous forme d'un objet JSON strict avec exactement ces clés : \
{\
  \"is_correct\": booléen,\
  \"score\": entier de 0 à 100,\
  \"explanation\": \"explication claire en français de l'erreur ou de la réussite\",\
  \"correction\": \"la bonne réponse ou null si c'était correct\"\
}\
Ne mets aucun texte explicatif avant ou après le JSON. N'utilise pas de blocs markdown.";

        let user_content = format!(
            "Mot/Expression japonaise à deviner: \"{}\"\n\
            Réponse de l'utilisateur: \"{}\"\n\
            Context (facultatif): \"{}\"\n\n\
            Évalue cette réponse et fournis le JSON exact.",
            req.vocab,
            req.user_answer,
            req.context.as_deref().unwrap_or("")
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_content,
            },
        ];

        let model_to_use = req.model.clone().unwrap_or_else(|| self.model.clone());

        let temp_to_use = req.temperature.unwrap_or(0.1);

        let chat_request = ChatRequest {
            model: model_to_use,
            messages,
            temperature: temp_to_use,
            top_p: req.top_p,
            max_tokens: req.max_tokens,
            frequency_penalty: req.frequency_penalty,
        };

        let mut request_builder = self.client.post(&endpoint).json(&chat_request);

        let api_key_to_use = req.api_key.clone().or_else(|| env::var("LLM_API_KEY").ok());
        if let Some(ref key) = api_key_to_use
            && !key.trim().is_empty()
        {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| format!("Erreur réseau lors de l'appel au LLM local: {}", e))?;

        if !response.status().is_success() {
            let error_status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!(
                "Le serveur LLM a répondu avec l'erreur {}: {}",
                error_status, error_text
            ));
        }

        let chat_resp: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Impossible de décoder la réponse du LLM: {}", e))?;

        if chat_resp.choices.is_empty() {
            return Err("Le LLM a retourné une réponse vide (pas de choix).".to_string());
        }

        let raw_content = &chat_resp.choices[0].message.content;
        let cleaned_json = self.clean_json_response(raw_content);

        let eval: EvaluateResponse = serde_json::from_str(&cleaned_json).map_err(|e| {
            format!(
                "Échec du parsing JSON de la réponse LLM. Reçu: \"{}\". Erreur: {}",
                raw_content, e
            )
        })?;

        Ok(eval)
    }

    pub async fn generate_hint(&self, req: &HintRequest) -> Result<HintResponse, String> {
        let api_url_to_use = req.api_url.clone().unwrap_or_else(|| self.api_url.clone());
        let endpoint = format!("{}/chat/completions", api_url_to_use.trim_end_matches('/'));
        let system_prompt = "Tu es un tuteur de japonais pour l'application LearnWithAnime. \
Donne un indice court pour aider l'utilisateur à deviner le mot ou l'expression japonaise demandé(e). \
Ne donne JAMAIS directement le mot ou sa traduction complète. \
L'indice dépend du niveau (tier) demandé : \
- Tier 1 : Donne un indice sur la forme du kanji (nombre de traits, radicaux) ou sa lecture phonétique (kana/furigana), sans donner le sens en français. \
- Tier 2 : Donne un exemple de phrase simple en japonais qui utilise ce mot, avec la traduction française de la phrase (mais cache le mot japonais cible par un blanc ou [____]), afin de comprendre son contexte d'usage. \
Sois très concis (1-2 phrases maximum) et réponds directement en français.";

        let user_content = format!(
            "Mot/Expression japonaise : \"{}\"\n\
            Indice demandé : Tier {} (1=forme/lecture, 2=phrase d'exemple dans un contexte)",
            req.vocab, req.tier
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_content,
            },
        ];

        let model_to_use = req.model.clone().unwrap_or_else(|| self.model.clone());

        let temp_to_use = req.temperature.unwrap_or(0.7);

        let chat_request = ChatRequest {
            model: model_to_use,
            messages,
            temperature: temp_to_use,
            top_p: req.top_p,
            max_tokens: req.max_tokens,
            frequency_penalty: req.frequency_penalty,
        };

        let mut request_builder = self.client.post(&endpoint).json(&chat_request);

        let api_key_to_use = req.api_key.clone().or_else(|| env::var("LLM_API_KEY").ok());
        if let Some(ref key) = api_key_to_use
            && !key.trim().is_empty()
        {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| format!("Erreur réseau lors de l'appel au LLM local: {}", e))?;

        if !response.status().is_success() {
            let error_status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!(
                "Le serveur LLM a répondu avec l'erreur {}: {}",
                error_status, error_text
            ));
        }

        let chat_resp: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Impossible de décoder la réponse du LLM: {}", e))?;

        if chat_resp.choices.is_empty() {
            return Err("Le LLM a retourné une réponse vide (pas de choix).".to_string());
        }

        let content = chat_resp.choices[0].message.content.trim().to_string();

        Ok(HintResponse { hint: content })
    }

    /// Récupère la liste des modèles disponibles sur l'API du LLM local
    pub async fn get_models(&self, req: &ModelsRequest) -> Result<Vec<String>, String> {
        let api_url_to_use = req.api_url.clone().unwrap_or_else(|| self.api_url.clone());
        let endpoint = format!("{}/models", api_url_to_use.trim_end_matches('/'));

        let mut request_builder = self.client.get(&endpoint);

        let api_key_to_use = req.api_key.clone().or_else(|| env::var("LLM_API_KEY").ok());
        if let Some(ref key) = api_key_to_use
            && !key.trim().is_empty()
        {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| format!("Erreur réseau lors de la récupération des modèles: {}", e))?;

        if !response.status().is_success() {
            let error_status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!(
                "Le serveur LLM a répondu avec l'erreur {} lors de la récupération des modèles: {}",
                error_status, error_text
            ));
        }

        let model_list: ModelListResponse = response
            .json()
            .await
            .map_err(|e| format!("Impossible de décoder la liste des modèles: {}", e))?;

        let mut ids: Vec<String> = model_list.data.into_iter().map(|item| item.id).collect();
        ids.sort();

        Ok(ids)
    }
}
