# Dataset per Modello Locale Claude Code — Istruzioni e Ricerca Web

> Stato: 2026-08-11 · Obiettivo: addestrare un modello 4B-7B che entra in 8GB VRAM (RTX 2070), bilingue IT+EN, con function calling affidabile per Claude Code.

---

## 1. Il problema che risolviamo

I modelli locali attuali sulla 2070 (8GB) falliscono nel tool calling di Claude Code:

| Modello | Dimensione | Tool calling | Problema |
|---|---|---|---|
| gemma4-e4b-claude-coder | 9.5GB | ? (troppo grande) | non entra in 8GB |
| gemma-4-E4B standard | 4.6GB | 1 su 3 | genera reasoning invece del tool_call |
| qwen2.5-coder-7b | 4.4GB | ❌ | emette il JSON come testo |

**L'obiettivo**: un modello che, data una richiesta con tool definitions, emette SEMPRE il `tool_call` strutturato (come Gemini o nemotron-ultra fanno) — non una descrizione testuale.

---

## 2. Requisiti del modello

- **Dimensioni**: 4B-7B parametri, quantizzato Q4_K_M → 3-5GB, entra in 8GB con context 8-32k
- **Lingue**: italiano (primario per Alfonso) + inglese (tecnico)
- **Capacità**:
  - Chitchat e conversazione naturale (IT+EN)
  - Coding base (refactor, spiegazioni, debug)
  - Reasoning (analisi, architettura)
  - **Function calling**: emettere `{"name": "...", "arguments": {...}}` strutturato in formato OpenAI/Anthropic
- **Formato output**: conforme a chat/completions con `tool_calls` (OpenAI) e `tool_use` (Anthropic)

---

## 3. Struttura del dataset

### 3.1 Funzioni base (40% del dataset)
- Chitchat IT+EN: conversazione quotidiana, saluti, domande semplici
- Coding: istruzioni di codice, spiegazioni, fix di bug, refactor
- Reasoning: analisi, confronti, pianificazione, domande "perché"

**Formato**: conversazioni multi-turno in formato chat
```json
{"messages": [
  {"role": "system", "content": "Sei un assistente utile."},
  {"role": "user", "content": "Spiega cos'è un gateway LLM"},
  {"role": "assistant", "content": "Un gateway LLM è ..."}
]}
```

### 3.2 Function calling (40% del dataset) — IL CUORE
- Richieste che richiedono uno strumento (Bash, Read, Write, WebSearch, AskUserQuestion, Edit...)
- Il modello deve emettere il `tool_call` strutturato, NON spiegare

**Formato OpenAI** (per il training):
```json
{"messages": [
  {"role": "user", "content": "Esegui ls nella directory"},
  {"role": "assistant", "content": null,
   "tool_calls": [{"id": "call_1", "type": "function",
     "function": {"name": "Bash", "arguments": "{\"command\": \"ls\"}"}}]}
]}
```

**Casi da coprire**:
- Chiamata singola
- Chiamate multiple in parallelo
- Tool con argomenti complessi (nested JSON)
- Tool che falliscono → tool_result con errore → nuova chiamata o risposta
- Cicli: tool_use → tool_result → altra tool_call → risposta finale
- Richiesta ambigua → AskUserQuestion per chiarire
- Negazione: "non usare tool, rispondi direttamente"

### 3.3 Adattamento bilingue dei tool (20%)
- Stesso esempio in italiano e inglese (per imparare che il tool call è indipendente dalla lingua)
- System prompt in IT che chiede tool → tool_call in inglese (nomi tool sempre in inglese)

---

## 4. Istruzioni precise per la RICERCA WEB

Cerca e salva i risultati in `docs/TRAINING_RESEARCH.md`:

### 4.1 Dataset esistenti di function calling (da cercare su HuggingFace/GitHub)
- `glaive-function-calling-v2` — dataset standard di function calling
- `fireworks-function-calling` — esempi di tool call
- `berkeley-function-call-leaderboard` (BFCL) — benchmark, non dataset ma utile
- `toolbench` — dataset di tool use
- `xlam-function-calling-60k` — 60k esempi multilingue (verificare se include IT)
- Cercare: "function calling dataset italian", "tool calling dataset small model"
- Per ogni dataset: nome, dimensione, licenza, lingua, formato

### 4.2 Modelli base bilingue IT+EN 4B-7B (da cercare)
- `Qwen2.5-7B-Instruct` — buon base, supporta IT
- `Mistral-7B-Instruct` — buon base
- `gemma-4-E4B` / `gemma-3-4B` — base attuale
- `Ministral-8B` / `Llama-3.1-8B` — candidati
- Verificare: supporto italiano, licenza, qualità tool calling di base

### 4.3 Tecniche di fine-tuning (da cercare)
- **QLoRA** su 8GB VRAM: fattibile per 7B? (cercare "QLoRA 7B 8GB VRAM")
- **LoRA** standard su 2070
- **Unsloth** — tool di training ottimizzato per GPU consumer
- **Axolotl** — config YAML per fine-tuning
- Full fine-tune vs LoRA: cosa serve per tool calling
- Cercare: "fine-tune small model function calling", "LoRA tool calling 4B"

### 4.4 Prompt format per tool calling (da cercare)
- Il formato esatto di training per far emettere tool_calls strutturati
- `Hermes 2 Pro` format (JSON schema nel system prompt)
- `functionary` — modello specializzato in function calling (base da studiare)
- `gorilla` — altro modello function calling

### 4.5 Risorse per il training
- GPU: 2070 8GB è sufficiente per QLoRA su 4B? (verificare VRAM per training)
- Alternative: Google Colab free, cloud, il nodo con più VRAM (hkstyle ~120GB)
- Tempo stimato per QLoRA su 4B con 10k esempi

---

## 5. Metriche di successo

- **Tool call corretto ≥ 90%**: su un test di 100 richieste con tool, ≥90 emettono `tool_calls` strutturati validi
- **Zero "JSON come testo"**: mai emettere il JSON del tool nel content
- **Chitchat preservato**: qualità conversazione non degradata (valutazione soggettiva + test)
- **Bilingue**: risponde correttamente in IT e EN

---

## 6. Piano di esecuzione

1. **Ricerca** (4.1-4.5) → salvare in `docs/TRAINING_RESEARCH.md`
2. **Selezione base**: scegliere il modello base 4B-7B bilingue più adatto
3. **Costruzione dataset**: generare esempi di function calling (si può usare Gemini/nemotron-ultra come "teacher" per creare coppie domanda→tool_call, poi verificarle)
4. **Training**: QLoRA/LoRA con il dataset
5. **Valutazione**: test sul nodo 2070 con il benchmark di tool calling
6. **Iterazione**: aggiungere esempi dove fallisce

---

## 7. Note importanti

- **La licenza**: verificare che dataset e modello base permettano il fine-tuning (la maggior parte sono Apache-2.0 o MIT, ma Qwen è Apache, gemma è Gemma license)
- **Il dataset va costruito con cura**: 10k esempi di qualità > 100k di rumore
- **Usare Gemini/nemotron-ultra come teacher**: generare le coppie (richiesta → tool_call corretto) e validarle, è il modo più rapido per costruire il dataset senza scrivere a mano
- **La 2070 può fare QLoRA su 4B**: verificare, altrimenti usare il nodo hkstyle (~120GB VRAM)
