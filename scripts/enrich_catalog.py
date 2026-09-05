import json

METADATA_MAP = {
    "groq": {
        "signup_url": "https://console.groq.com/keys",
        "free_tier_info": "Free Tier permanente generoso: 30 RPM e fino a 14.400 richieste/giorno su gpt-oss-120b e Qwen 3.8. Nessuna carta di credito richiesta, login diretto con Google/GitHub.",
        "category": "⚡ LPUs & Ultra-Fast Inference",
        "default_model": "openai/gpt-oss-120b",
        "base_url": "https://api.groq.com/openai/v1"
    },
    "gemini": {
        "signup_url": "https://aistudio.google.com/app/apikey",
        "free_tier_info": "Google AI Studio offre 15 RPM e 1.500 richieste/giorno gratuite per sempre sui modelli Flash. Basta un account Google personale, nessuna fatturazione cloud necessaria.",
        "category": "♊ Google AI Studio",
        "default_model": "gemini-3.5-flash-lite",
        "base_url": "https://generativelanguage.googleapis.com/v1beta/openai"
    },
    "sambanova": {
        "signup_url": "https://cloud.sambanova.ai/apis",
        "free_tier_info": "Inference ad altissima velocità su chip SN40L. Tier Developer gratuito con quota oraria ripristinata continuamente su Llama 3.3 e Qwen. Login con GitHub.",
        "category": "⚡ LPUs & Ultra-Fast Inference",
        "default_model": "gpt-oss-120b",
        "base_url": "https://api.sambanova.ai/v1"
    },
    "cerebras": {
        "signup_url": "https://cloud.cerebras.ai/",
        "free_tier_info": "Wafer-Scale Engine da oltre 2000 token/sec. Tier free iniziale con $5 di crediti all iscrizione per Llama 3.3 e gpt-oss.",
        "category": "⚡ LPUs & Ultra-Fast Inference",
        "default_model": "gpt-oss-120b",
        "base_url": "https://api.cerebras.ai/v1"
    },
    "mistral": {
        "signup_url": "https://console.mistral.ai/api-keys/",
        "free_tier_info": "Tier sperimentale gratuito \"La Plateforme\" per Mistral Nemo, Codestral e Pixtral. Accesso immediato con account GitHub/Google.",
        "category": "🌪️ European Frontier Labs",
        "default_model": "mistral-code-latest",
        "base_url": "https://api.mistral.ai/v1"
    },
    "openrouter": {
        "signup_url": "https://openrouter.ai/keys",
        "free_tier_info": "Aggregatore universale con decine di modelli con suffisso \":free\" a costo zero (Minimax M2.7, Gemini Free, Qwen Free). Registrazione con Google o wallet Web3.",
        "category": "🪐 Cloud Aggregators",
        "default_model": "minimax/minimax-m2.7:free",
        "base_url": "https://openrouter.ai/api/v1"
    },
    "fireworks": {
        "signup_url": "https://fireworks.ai/api-keys",
        "free_tier_info": "Inference serverless ultra-veloce con $1 di credito omaggio all iscrizione (equivalente a milioni di token). Login con Google/GitHub.",
        "category": "🔥 Serverless Inference",
        "default_model": "accounts/fireworks/models/llama-v3p3-70b-instruct",
        "base_url": "https://api.fireworks.ai/inference/v1"
    },
    "inception": {
        "signup_url": "https://inceptionlabs.ai/",
        "free_tier_info": "Accesso di ricerca e API gateway veloce per modelli Mercury e community open weights.",
        "category": "🔥 Serverless Inference",
        "default_model": "mercury-2",
        "base_url": "https://api.inceptionlabs.ai/v1"
    },
    "deepseek": {
        "signup_url": "https://platform.deepseek.com/api_keys",
        "free_tier_info": "Frontier lab open weights: DeepSeek-V3 e DeepSeek-R1 con 5 milioni di token gratuiti di prova all attivazione dell account.",
        "category": "🧠 Reasoning & Frontier Labs",
        "default_model": "deepseek-chat",
        "base_url": "https://api.deepseek.com/v1"
    },
    "nvidia": {
        "signup_url": "https://build.nvidia.com/",
        "free_tier_info": "NVIDIA NIM Developer Program: 1.000 crediti API gratuiti permanenti per testare modelli su cluster H100/H200 (Llama 3.3, Gemma, Mistral, Qwen).",
        "category": "🟢 NVIDIA Enterprise NIM",
        "default_model": "meta/llama-3.3-70b-instruct",
        "base_url": "https://integrate.api.nvidia.com/v1"
    },
    "together": {
        "signup_url": "https://api.together.ai/settings/api-keys",
        "free_tier_info": "$5 di crediti di prova gratuiti all iscrizione per tutti i modelli open-source (Llama 3.3, Qwen 2.5 Coder, DeepSeek R1).",
        "category": "🤝 Open Source Cloud",
        "default_model": "meta-llama/Llama-3.3-70B-Instruct-Turbo",
        "base_url": "https://api.together.xyz/v1"
    },
    "cohere": {
        "signup_url": "https://dashboard.cohere.com/api-keys",
        "free_tier_info": "Trial Key gratuita permanente con limite di 1000 chiamate/mese per Command R+ e modelli di embedding.",
        "category": "🧠 Enterprise Frontier Labs",
        "default_model": "command-r-plus-08-2024",
        "base_url": "https://api.cohere.com/v2"
    },
    "huggingface": {
        "signup_url": "https://huggingface.co/settings/tokens",
        "free_tier_info": "Accesso gratuito a Hugging Face Serverless Inference API per migliaia di modelli open source usando un User Access Token personale gratuito.",
        "category": "🤗 Community & Open Weights",
        "default_model": "meta-llama/Llama-3.3-70B-Instruct",
        "base_url": "https://api-inference.huggingface.co/v1"
    },
    "novita": {
        "signup_url": "https://novita.ai/settings/key-management",
        "free_tier_info": "Credito gratuito di benvenuto all iscrizione per Llama 3.3, DeepSeek e modelli grafici.",
        "category": "⚡ Serverless Inference",
        "default_model": "meta-llama/llama-3.3-70b-instruct",
        "base_url": "https://api.novita.ai/v3/openai"
    },
    "hyperbolic": {
        "signup_url": "https://app.hyperbolic.xyz/settings",
        "free_tier_info": "Piattaforma di compute decentralizzato: crediti gratuiti iniziali per Llama 3.3 70B e DeepSeek R1.",
        "category": "⚡ Serverless Inference",
        "default_model": "meta-llama/Llama-3.3-70B-Instruct",
        "base_url": "https://api.hyperbolic.xyz/v1"
    },
    "deepinfra": {
        "signup_url": "https://deepinfra.com/dash/api_keys",
        "free_tier_info": "$1.80 di crediti gratuiti all iscrizione con GitHub (fino a 10M di token su modelli compatti).",
        "category": "⚡ Serverless Inference",
        "default_model": "meta-llama/Meta-Llama-3.3-70B-Instruct",
        "base_url": "https://api.deepinfra.com/v1/openai"
    },
    "siliconflow": {
        "signup_url": "https://cloud.siliconflow.cn/account/ak",
        "free_tier_info": "20 milioni di token gratuiti all attivazione (Qwen 2.5 Coder, DeepSeek V3/R1, GLM). Registrazione con email/GitHub.",
        "category": "🐉 Asian Frontier AI",
        "default_model": "Qwen/Qwen2.5-Coder-32B-Instruct",
        "base_url": "https://api.siliconflow.cn/v1"
    },
    "ai21": {
        "signup_url": "https://studio.ai21.com/account/api-key",
        "free_tier_info": "Tier gratuito iniziale con $10 di crediti per i modelli Jamba 1.5 Mini e Large (architettura ibrida SSM-Transformer).",
        "category": "🧠 Enterprise Frontier Labs",
        "default_model": "jamba-1.5-mini",
        "base_url": "https://api.ai21.com/studio/v1"
    },
    "opencode": {
        "signup_url": "https://opencode.ai/zen",
        "free_tier_info": "Zero-Auth nativo: endpoint pubblico libero per modelli Mimo v2.5 e coding assistance.",
        "category": "🆓 Zero-Auth Free",
        "default_model": "mimo-v2.5-free",
        "base_url": "https://opencode.ai/zen/v1"
    },
    "pollinations": {
        "signup_url": "https://pollinations.ai/",
        "free_tier_info": "Completamente gratuito e aperto a tutti senza registrazione nè chiavi API. Supporta modelli testuali e multimodali open-source.",
        "category": "🆓 Zero-Auth Free",
        "default_model": "openai",
        "base_url": "https://text.pollinations.ai/openai"
    },
    "agnes": {
        "signup_url": "https://agnes-ai.com/",
        "free_tier_info": "Cloud inference gateway Singapore per modelli multimodali leggeri della famiglia Agnes AI.",
        "category": "🐉 Asian Frontier AI",
        "default_model": "agnes-2.5-flash",
        "base_url": "https://apihub.agnes-ai.com/v1"
    }
}

with open("data/free_providers_catalog.json") as f:
    catalog = json.load(f)

for entry in catalog:
    eid = entry["id"]
    if entry["base_url"].endswith("/chat/completions"):
        entry["base_url"] = entry["base_url"].replace("/chat/completions", "")
    
    pname = entry.get("name", eid)
    if eid in METADATA_MAP:
        meta = METADATA_MAP[eid]
        entry["signup_url"] = meta["signup_url"]
        entry["free_tier_info"] = meta["free_tier_info"]
        entry["category"] = meta["category"]
        if "default_model" in meta:
            entry["default_model"] = meta["default_model"]
        if "base_url" in meta:
            entry["base_url"] = meta["base_url"]
    else:
        domain = entry["base_url"].split("/")[2] if "://" in entry["base_url"] else eid
        entry["signup_url"] = f"https://{domain}"
        entry["free_tier_info"] = f"Tier gratuito per sviluppatori su {pname}. Registrati su {domain} per generare la chiave API."
        entry["category"] = "🌐 Cloud AI Provider"

for entry in catalog:
    if entry["id"] == "auggie":
        entry["id"] = "together"
        entry["name"] = "Together AI"
        entry["base_url"] = "https://api.together.xyz/v1"
        entry["format"] = "openai"
        entry["auth_type"] = "bearer"
        entry["env_var"] = "TOGETHER_API_KEY"
        entry["no_auth"] = False
        entry["default_model"] = "meta-llama/Llama-3.3-70B-Instruct-Turbo"
        entry["models"] = ["meta-llama/Llama-3.3-70B-Instruct-Turbo", "deepseek-ai/DeepSeek-R1", "Qwen/Qwen2.5-Coder-32B-Instruct"]
        entry["signup_url"] = "https://api.together.ai/settings/api-keys"
        entry["free_tier_info"] = "Credito di benvenuto di $5 per tutti i modelli open source senza carta richiesta."
        entry["category"] = "🤝 Open Source Cloud"
    elif entry["id"] == "zcode":
        entry["id"] = "cloudflare"
        entry["name"] = "Cloudflare Workers AI"
        entry["base_url"] = "https://api.cloudflare.com/client/v4/accounts"
        entry["format"] = "openai"
        entry["auth_type"] = "bearer"
        entry["env_var"] = "CLOUDFLARE_API_TOKEN"
        entry["no_auth"] = False
        entry["default_model"] = "@cf/meta/llama-3.3-70b-instruct-fp8-fast"
        entry["signup_url"] = "https://dash.cloudflare.com/"
        entry["free_tier_info"] = "10.000 neuroni gratuiti al giorno (~100k token) per sempre con account Cloudflare gratuito."
        entry["category"] = "☁️ Edge Compute"
    elif entry["id"] == "chipotle":
        entry["id"] = "github_models"
        entry["name"] = "GitHub Models Free Tier"
        entry["base_url"] = "https://models.inference.ai.azure.com"
        entry["format"] = "openai"
        entry["auth_type"] = "bearer"
        entry["env_var"] = "GITHUB_TOKEN"
        entry["no_auth"] = False
        entry["default_model"] = "gpt-4o-mini"
        entry["models"] = ["gpt-4o-mini", "Meta-Llama-3.3-70B-Instruct", "Mistral-large-2411", "Phi-4"]
        entry["signup_url"] = "https://github.com/marketplace/models"
        entry["free_tier_info"] = "Accesso gratuito con rate limit giornaliero a GPT-4o-mini, Llama 3.3 e Phi-4 usando il tuo Personal Access Token di GitHub."
        entry["category"] = "🐙 GitHub / Microsoft Azure"

with open("data/free_providers_catalog.json", "w") as f:
    json.dump(catalog, f, indent=2)

print("Catalogo arricchito con successo!")
