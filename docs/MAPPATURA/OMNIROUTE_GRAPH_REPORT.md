# Graph Report - .  (2026-08-10)

## Corpus Check
- Corpus is ~17,051 words - fits in a single context window. You may not need a graph.

## Summary
- 302 nodes · 515 edges · 17 communities (15 shown, 2 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 3 edges (avg confidence: 0.5)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Assessment & Model Evaluation
- Cost Rules & Budget
- Fallback Policy
- Quota Cache
- Combo Repositories
- Pipeline Execution
- Config Audit
- Provider Expiration
- Degradation
- Response Meta Headers
- Connection Model Rules
- API Responses
- Tag Router
- Model Availability
- Combo Resolver

## God Nodes (most connected - your core abstractions)
1. `ModelAssessment` - 18 edges
2. `getState()` - 14 edges
3. `Assessor` - 11 edges
4. `SelfHealer` - 10 edges
5. `ComboRepository` - 10 edges
6. `hydrateQuotaCacheFromSnapshots()` - 9 edges
7. `AssessmentConfig` - 8 edges
8. `checkBudget()` - 8 edges
9. `buildOmniRouteResponseMetaHeaders()` - 8 edges
10. `getQuotaWindowStatus()` - 8 edges

## Surprising Connections (you probably didn't know these)
- `evaluateRequest()` --calls--> `checkBudget()`  [EXTRACTED]
  src/domain/policyEngine.ts → src/domain/costRules.ts
- `ProbeResult` --references--> `AssessmentStatus`  [EXTRACTED]
  src/domain/assessment/assessor.ts → src/domain/assessment/types.ts
- `Assessor` --references--> `AssessmentConfig`  [EXTRACTED]
  src/domain/assessment/assessor.ts → src/domain/assessment/types.ts
- `Assessor` --references--> `ModelAssessment`  [EXTRACTED]
  src/domain/assessment/assessor.ts → src/domain/assessment/types.ts
- `SelfHealer` --references--> `AssessmentConfig`  [EXTRACTED]
  src/domain/assessment/selfHealer.ts → src/domain/assessment/types.ts

## Import Cycles
- None detected.

## Communities (17 total, 2 thin omitted)

### Community 0 - "Assessment & Model Evaluation"
Cohesion: 0.11
Nodes (27): Assessor, percentile(), ProbeResult, Categorizer, CATEGORY_WEIGHTS, MODEL_PATTERNS, TIER_SCORES, Combo (+19 more)

### Community 1 - "Cost Rules & Budget"
Cohesion: 0.11
Nodes (30): BudgetConfig, BudgetResetInterval, budgets, BudgetSummary, BudgetWindow, checkBudget(), CostEntry, emitBudgetWarning() (+22 more)

### Community 2 - "Fallback Policy"
Cohesion: 0.09
Nodes (21): ensureLoaded(), fallbackChains, getAllFallbackChains(), getNextFallback(), hasFallback(), registerFallback(), removeFallback(), resolveFallbackChain() (+13 more)

### Community 3 - "Quota Cache"
Cohesion: 0.14
Nodes (32): advancedWindowResetAt(), backgroundRefreshTick(), clampPercent(), __clearForTests(), earliestResetAt(), getQuotaCache(), getQuotaCacheStats(), getQuotaWindowStatus() (+24 more)

### Community 4 - "Combo Repositories"
Cohesion: 0.08
Nodes (9): ComboRecord, ComboReorderResult, ComboRepository, ComboUpdateResult, CreateModelComboMappingInput, ModelComboMapping, ModelComboMappingPage, ModelComboMappingRepository (+1 more)

### Community 5 - "Pipeline Execution"
Cohesion: 0.11
Nodes (21): executePipeline(), executeStage(), FitnessTier, parseReflectJson(), PipelineConfig, PipelineResult, PipelineStage, ReflectFail (+13 more)

### Community 6 - "Config Audit"
Cohesion: 0.14
Nodes (12): AuditAction, auditLog, AuditSource, AuditTarget, computeDiff(), ConfigAuditEntry, ConfigDiff, ConfigSnapshot (+4 more)

### Community 7 - "Provider Expiration"
Cohesion: 0.18
Nodes (11): calculateStatus(), expirations, ExpirationSummary, ExpiryStatus, ExpiryType, getAllExpirations(), getExpiration(), getExpirationSummary() (+3 more)

### Community 8 - "Degradation"
Cohesion: 0.18
Nodes (7): DegradationLevel, DegradationStatus, DegradedResult, registry, updateRegistry(), withDegradation(), withDegradationSync()

### Community 9 - "Response Meta Headers"
Cohesion: 0.33
Nodes (12): attachOmniRouteMetaHeaders(), attachOmniRouteMetaToResponse(), buildOmniRouteDecisionHeaderValue(), buildOmniRouteResponseMetaHeaders(), buildOmniRouteSseMetadataComment(), formatOmniRouteCost(), getOmniRouteTokenCounts(), toFiniteNumber() (+4 more)

### Community 10 - "Connection Model Rules"
Cohesion: 0.29
Nodes (11): asRecord(), ConnectionLike, getConnectionExcludedModels(), getModelMatchCandidates(), hasEligibleConnectionForModel(), isModelExcludedByConnection(), JsonRecord, normalizeExcludedModelPatterns() (+3 more)

### Community 11 - "API Responses"
Cohesion: 0.33
Nodes (7): apiErrorResponse(), badRequest(), conflict(), forbidden(), internalError(), notFound(), unauthorized()

### Community 12 - "Tag Router"
Cohesion: 0.33
Nodes (8): asRecord(), getConnectionRoutingTags(), JsonRecord, normalizeRoutingTagMatchMode(), normalizeRoutingTags(), normalizeSingleRoutingTag(), resolveRequestRoutingTags(), RoutingTagMatchMode

## Knowledge Gaps
- **66 isolated node(s):** `TIER_SCORES`, `CATEGORY_WEIGHTS`, `MODEL_PATTERNS`, `ComboModel`, `Combo` (+61 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What connects `TIER_SCORES`, `CATEGORY_WEIGHTS`, `MODEL_PATTERNS` to the rest of the system?**
  _66 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Assessment & Model Evaluation` be split into smaller, more focused modules?**
  _Cohesion score 0.11020408163265306 - nodes in this community are weakly interconnected._
- **Should `Cost Rules & Budget` be split into smaller, more focused modules?**
  _Cohesion score 0.11229946524064172 - nodes in this community are weakly interconnected._
- **Should `Fallback Policy` be split into smaller, more focused modules?**
  _Cohesion score 0.0873440285204991 - nodes in this community are weakly interconnected._
- **Should `Quota Cache` be split into smaller, more focused modules?**
  _Cohesion score 0.14204545454545456 - nodes in this community are weakly interconnected._
- **Should `Combo Repositories` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._
- **Should `Pipeline Execution` be split into smaller, more focused modules?**
  _Cohesion score 0.11231884057971014 - nodes in this community are weakly interconnected._