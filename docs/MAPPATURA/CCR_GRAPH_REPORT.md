# Graph Report - .  (2026-08-10)

## Corpus Check
- Large corpus: 285 files · ~1,379,144 words. Semantic extraction will be expensive (many Claude tokens). Consider running on a subfolder.

## Summary
- 5919 nodes · 16935 edges · 187 communities (181 shown, 6 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 227 edges (avg confidence: 0.61)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- contracts app default
- gateway features hosted
- observability request log
- gateway http io
- providers presets utils
- web management server
- gateway features hosted
- providers runtime topology
- profiles launch service
- proxy system proxy
- agents claude app
- src config config
- observability request log
- contracts app gatewayprovidercapability
- gateway claude code
- gateway features context
- observability request log
- mcp fusion vision
- contracts deep link
- gateway internal shared
- agents zcode profile
- agents local providers
- observability request log
- agents local providers
- config config clampnumber
- gateway upstream executor
- proxy service proxyservice
- mcp toolhub mcp
- agents local providers
- gateway features model
- observability raw trace
- gateway core runtime
- agents codex app
- agents local providers
- profiles launch core
- providers model catalog
- agents codex model
- agents zcode profile
- platform windows app
- agents bot gateway
- contracts app gateway
- providers probe codexaccesstokenexpired
- agents claude app
- contracts app gatewaypluginappconfig
- gateway context archive
- gateway core runtime
- providers account service
- profiles launch service
- observability route trace
- agents kilo profile
- gateway application gateway
- models pricing service
- observability request log
- agents local providers
- media service mediaservice
- gateway context archive
- mcp fusion config
- agents claude app
- config config repository
- test unit config
- config config repository
- test unit gateway
- routing route script
- routing route script
- gateway remote control
- media executors gatewaymediaexecutor
- mcp tool discovery
- proxy service proxyservice
- providers account service
- contracts app usagecomparisonrow
- agents opencode profile
- mcp toolhub config
- observability raw trace
- proxy system proxy
- agents bot gateway
- providers new api
- gateway model catalog
- media service mediaservice
- mcp toolhub mcp
- observability request log
- plugins backend service
- profiles service buildcodexconfigtoml
- providers account service
- agents zcode profile
- mcp toolhub mcp
- plugins service gatewaypluginservice
- mcp media tools
- providers account service
- agents claude code
- config constants proxy
- routing config compiler
- gateway features codex
- media service mediaservice
- agents claude code
- gateway claude code
- mcp grok media
- config config bundledruntimepluginmodulecandi
- mcp toolhub mcp
- observability request log
- usage billing sync
- agents claude app
- mcp network capture
- observability request log
- agents codex media
- agents local providers
- config constants provider
- mcp fusion tool
- routing route script
- agents bot gateway
- agents codex model
- config config repository
- config constants legacy
- benchmark request log
- agents pi profile
- contracts app usagestatsrange
- plugins backend service
- routing route script
- usage store asnumber
- test integration mcp
- observability request log
- mcp toolhub mcp
- routing policy engine
- profiles api key
- gateway core runtime
- media storage mediaartifactstore
- web management server
- test integration gateway
- routing execution plan
- providers account service
- usage store usagestore
- platform socket compat
- mcp browser web
- media storage mediajobstore
- package scripts test
- agents cdp client
- test unit providers
- proxy service proxyservice
- gateway http request
- gateway context archive
- mcp toolhub mcp
- runtime app paths
- package dependencies the
- test unit agents
- test integration mcp
- gateway context archive
- gateway core runtime
- gateway context archive
- plugins service gatewaypluginservice
- test unit agents
- config config assertprovideraccountapikeytarg
- src contracts i18n
- mcp toolhub mcp
- test unit providers
- test integration agents
- agents zcode model
- mcp network capture
- plugins service gatewaypluginservice
- plugins service gatewaypluginservice
- profiles service clearglobalprofiletakeoverma
- providers account service
- src usage normalization
- test integration mcp
- test unit config
- ref
- config onboarding state
- mcp toolhub mcp
- models catalog file
- benchmark request log
- agents cdp client
- routing failure classifier
- mcp toolhub mcp
- test unit config
- gateway runtime change
- providers account service
- test integration mcp
- usage store buildusagewhereclause
- platform windows system
- test architecture gateway
- contracts app grok

## God Nodes (most connected - your core abstractions)
1. `isRecord()` - 181 edges
2. `stringValue()` - 159 edges
3. `AppConfig` - 74 edges
4. `isObject()` - 60 edges
5. `RequestLogRuntime` - 44 edges
6. `MediaService` - 43 edges
7. `isGatewayProviderEnabled()` - 40 edges
8. `normalizeRouteSelector()` - 40 edges
9. `readString()` - 39 edges
10. `formatError()` - 39 edges

## Surprising Connections (you probably didn't know these)
- `parseProfiles()` --indirect_call--> `appPath()`  [INFERRED]
  packages/core/src/config/config.ts → packages/core/src/agents/claude-app/gateway-service.ts
- `defaultClientModel()` --indirect_call--> `isGatewayProviderEnabled()`  [INFERRED]
  packages/core/src/agents/kilo/profile-config.ts → packages/core/src/contracts/app.ts
- `readKimiConfiguredProviders()` --indirect_call--> `baseUrl()`  [INFERRED]
  packages/core/src/agents/local-providers/kimi.ts → packages/core/test/integration/mcp/grok-media-service.test.mjs
- `readZcodeConfiguredProviders()` --indirect_call--> `baseUrl()`  [INFERRED]
  packages/core/src/agents/local-providers/zcode.ts → packages/core/test/integration/mcp/grok-media-service.test.mjs
- `restoreInactiveGlobalProfileConfigs()` --indirect_call--> `openCodeProviderId()`  [INFERRED]
  packages/core/src/profiles/service.ts → packages/core/src/agents/opencode/profile-config.ts

## Import Cycles
- 3-file cycle: `packages/core/src/agents/local-providers/codex.ts -> packages/core/src/proxy/system-proxy-fetch.ts -> packages/core/src/config/config.ts -> packages/core/src/agents/local-providers/codex.ts`
- 3-file cycle: `packages/core/src/agents/local-providers/grok.ts -> packages/core/src/proxy/system-proxy-fetch.ts -> packages/core/src/config/config.ts -> packages/core/src/agents/local-providers/grok.ts`
- 3-file cycle: `packages/core/src/routing/contracts.ts -> packages/core/src/routing/rewrite.ts -> packages/core/src/routing/model-registry.ts -> packages/core/src/routing/contracts.ts`

## Communities (187 total, 6 thin omitted)

### Community 0 - "contracts app default"
Cohesion: 0.02
Nodes (136): DEFAULT_PROXY_TARGETS, DefaultAppConfigOptions, AgentAnalysisConcurrencyPoint, AgentAnalysisSessionSelection, AgentAnalysisTracePayloadPart, AgentAnalysisTraceRunStatus, AppCaptureElementPngRequest, AppCaptureElementPngResult (+128 more)

### Community 1 - "gateway features hosted"
Cohesion: 0.05
Nodes (122): isLocalKimiOauthProviderPlugin(), openAiResponsesResponseFromSseEvents(), openAiResponsesSseFunctionCallIndex(), compactRecord(), isOpenAICompatChatCompletionsPath(), isSimplifiedCursorOpenAICompatChat(), normalizeCursorTool(), normalizeCursorToolChoice() (+114 more)

### Community 2 - "observability request log"
Cohesion: 0.03
Nodes (104): AgentAnalysisAgentRow, AgentAnalysisFilter, AgentAnalysisRequestRow, AgentAnalysisSessionDetail, AgentAnalysisSessionModelRow, AgentAnalysisSessionRow, AgentAnalysisSnapshot, AgentAnalysisSubagentRow (+96 more)

### Community 3 - "gateway http io"
Cohesion: 0.04
Nodes (74): iterations, legacyMs, optimizedMs, parsePasses, payload, runOptimizedPipeline(), RequestRouteTraceChange, codexCompactResponseStream() (+66 more)

### Community 4 - "providers presets utils"
Cohesion: 0.05
Nodes (69): ProviderAccountConfig, ProviderAccountMappingConfig, anthropicProviderPreset, bailianProviderPreset, claudeApiProviderPreset, code0ProviderPreset, deepSeekProviderAccountConfig, deepSeekProviderPreset (+61 more)

### Community 5 - "web management server"
Cohesion: 0.04
Nodes (94): LEGACY_API_KEYS_DB_FILES, AppDataExportResult, AppInfo, AppSaveConfigOptions, AppUpdateStatus, BotGatewayQrLoginWaitRequest, BotGatewayQrWindowOpenRequest, LocalAgentProviderImportRequest (+86 more)

### Community 6 - "gateway features hosted"
Cohesion: 0.06
Nodes (81): normalizeCoreGatewayVirtualModelProfiles(), appendAnthropicMessagesToolOutputs(), appendContextArchiveToolOutputs(), appendContextArchiveToolOutputsForTest(), appendOpenAiResponsesToolOutputs(), anthropicHostedWebSearchType(), claudeCodeWebSearchToolResultTexts(), extractAnthropicWebSearchQueryHint() (+73 more)

### Community 7 - "providers runtime topology"
Cohesion: 0.06
Nodes (83): GatewayProviderProtocol, isGatewayProviderEnabled(), addProviderNameVariants(), codexOauthLocalProviderNames(), compileCoreGatewayConfig(), compiledProviderNameForPlugin(), configuredAnthropicBetaDefault(), coreGatewayProviderSelectorName() (+75 more)

### Community 8 - "profiles launch service"
Cohesion: 0.05
Nodes (83): bundledNodePath(), CCR_CLI_COMPANION_RUNTIME_FILE_NAMES, CcrCliLauncherPreparation, chmodSafe(), claudeAppGatewayConfigFor(), claudeCodeApiKeyHelperFilename(), cleanupLegacyCcrCliLauncher(), cmdEnvValue() (+75 more)

### Community 9 - "proxy system proxy"
Cohesion: 0.06
Nodes (74): DATADIR, ProxyRuntimeConfig, ProxySystemStatus, windowsSystemCommand(), applyMacSystemProxy(), applySystemProxy(), applyWindowsSystemProxy(), applyWindowsWinHttpProxy() (+66 more)

### Community 10 - "agents claude app"
Cohesion: 0.05
Nodes (77): buildClaudeAppGatewayInferenceModels(), buildClaudeAppGatewayModelRoutes(), canonicalClaudeAppGatewayTargetModel(), claimClaudeAppGatewayRouteId(), claudeAppGatewayBaseDisplayName(), claudeAppGatewayDisplayNames(), claudeAppGatewayDisplayNameWithProvider(), claudeAppGatewayEncodedRouteId() (+69 more)

### Community 11 - "src config config"
Cohesion: 0.04
Nodes (76): appConfigWriteQueue, botGatewayWebSocketTransport(), claudeCodeProfileEnv(), codexCompatibleProfileEnv(), completeBotGatewayConfig(), dedupeTraySingletonWidgets(), DEFAULT_CONFIG, defaultBotGatewayAuthType() (+68 more)

### Community 12 - "observability request log"
Cohesion: 0.06
Nodes (41): RequestLogAdmission, maxRequestLogBodyBytes, AdmissionOperation, AdmissionOverlayEntry, boundedBuffer(), compactBoundedText(), constrainRawTraceFiles(), errorCode() (+33 more)

### Community 13 - "contracts app gatewayprovidercapability"
Cohesion: 0.05
Nodes (77): GatewayProviderCapability, GatewayProviderConnectivityCheckReport, GatewayProviderConnectivityCheckRequest, GatewayProviderProbeCandidate, GatewayProviderProbeCandidateResult, GatewayProviderProbeCandidatesRequest, GatewayProviderProbeProtocolResult, GatewayProviderProbeRequest (+69 more)

### Community 14 - "gateway claude code"
Cohesion: 0.06
Nodes (71): ProfileClientKind, RouterBuiltInAgentRuleId, RouterRuleCondition, appendDescriptionInstruction(), appendPromptSchemaDescriptionInstruction(), appendSystemInstruction(), appendToolDescriptionInstruction(), arrayElementMatches() (+63 more)

### Community 15 - "gateway features context"
Cohesion: 0.06
Nodes (71): contextArchiveConfigForApiKey(), contextArchiveMcpEnabled(), prepareContextArchiveRequest(), appendCompactHandoffTask(), codexCompactArchiveResponseContentType(), codexResponsesPathForCompact(), ContextArchiveResponseMode, hasCodexResponsesCompactionTrigger() (+63 more)

### Community 16 - "observability request log"
Cohesion: 0.05
Nodes (52): AgentAnalysisTracePayloadFullResult, AgentAnalysisTracePayloadRequest, RequestLogDetailRequest, RequestLogEntry, RequestLogFilterOptions, RequestLogListFilter, RequestLogPage, usagePriceCatalogNeedsRefresh() (+44 more)

### Community 17 - "mcp fusion vision"
Cohesion: 0.08
Nodes (67): analyzeVision(), analyzeWebSearch(), buildImageParts(), callTool(), clampInteger(), drainInputBuffer(), env(), extractProviderError() (+59 more)

### Community 18 - "contracts deep link"
Cohesion: 0.07
Nodes (66): ProviderAccountConnectorConfig, ProviderAccountMappedMeterConfig, ProviderAccountStandardConnectorConfig, ProviderDeepLinkPayload, ProviderDeepLinkRequest, ProviderManifestDeepLinkPayload, ProviderManifestFetchRequest, ProviderManifestFetchResult (+58 more)

### Community 19 - "gateway internal shared"
Cohesion: 0.06
Nodes (58): ApiKeyConfig, AppConfig, GatewayProviderCapabilityProtocol, ProviderCredentialConfig, VirtualModelFusionWebSearchProvider, apiKeyLimitRules(), authorize(), configuredApiKeys() (+50 more)

### Community 20 - "agents zcode profile"
Cohesion: 0.07
Nodes (59): resolveZcodeConfigFile(), CONFIGDIR, ccrManagedProfileDir(), claudeCodeApiKeyHelperCmdScript(), claudeCodeApiKeyHelperFilename(), claudeCodeApiKeyHelperShellScript(), claudeCodeGatewayEnvKeys, claudeCodeRemovedAuthEnvKeys (+51 more)

### Community 21 - "agents local providers"
Cohesion: 0.07
Nodes (58): attachCodexRateLimitResetCreditDetails(), codexAccountRateLimitMapping, codexAccountRateLimitResetCreditsMapping, codexAccountTokenUsageMapping, codexBackendRequestTransform(), codexCandidateWithProbedModels(), codexDateString(), codexDefaultModels (+50 more)

### Community 22 - "observability request log"
Cohesion: 0.06
Nodes (58): asString(), collectAnthropicStreamToolInput(), collectOpenAiStreamToolInput(), collectStreamedToolCallInput(), collectStreamedToolCallInputs(), collectToolCallPayloads(), collectToolCalls(), collectToolResultPayloads() (+50 more)

### Community 23 - "agents local providers"
Cohesion: 0.08
Nodes (54): grokOauthPlugin(), adoptPeerRotatedKimiAuth(), asciiHeader(), findKimiOauthProvider(), findTomlSection(), importKimiProvider(), kimiAccessTokenExpired(), kimiAuthPlugin() (+46 more)

### Community 24 - "config config clampnumber"
Cohesion: 0.09
Nodes (57): clampNumber(), hasConfigFileApiKeys(), hasUnsupportedNvidiaCapabilities(), isObject(), parseAgent(), parseApiKeyConfig(), parseApiKeyLimits(), parseApiKeys() (+49 more)

### Community 25 - "gateway upstream executor"
Cohesion: 0.09
Nodes (50): GatewayProviderConfig, parseJsonObjectSafe(), serializeJsonBodyWithModel(), clampNumber(), applyProviderCapabilityRouting(), buildAttemptBody(), buildUpstreamAttempts(), clearTargetProviderHeaders() (+42 more)

### Community 26 - "proxy service proxyservice"
Cohesion: 0.08
Nodes (48): ProxyForwardMode, ActiveProxyNetworkCapture, AttachedServer, bodyToDisplayText(), buildGatewayUrl(), CapturedHeaders, captureProxyError(), cloneHeaders() (+40 more)

### Community 27 - "mcp toolhub mcp"
Cohesion: 0.05
Nodes (56): buildExecutionPlanArgs(), buildTsDefinitions(), CatalogEntry, CodeToolSessionState, delay(), errorCode(), executionPlanInstructions, GatewayMcpRemoteServerConfig (+48 more)

### Community 28 - "agents local providers"
Cohesion: 0.07
Nodes (49): importGrokProviderWithAuth(), configuredOpenCodeApiKey(), configuredOpenCodeApiKeyIsPresent(), configuredOpenCodeApiKeyValue(), deepMergeRecords(), emptyOpenCodeProtocolRecord(), importOpenCodeProvider(), isGeneratedOpenCodeAccountConnector() (+41 more)

### Community 29 - "gateway features model"
Cohesion: 0.08
Nodes (50): VirtualModelProfileConfig, resolveGrokProfileRouteTarget(), rewriteModelSelectorForCoreGatewayProfile(), buildClaudeCodeDiscoverableModelIds(), buildClaudeCodeDiscoverableModels(), buildGatewayDiscoverableModelIds(), claudeCodeDiscoveryModelId(), claudeCodeOneMillionContextModelId() (+42 more)

### Community 30 - "observability raw trace"
Cohesion: 0.08
Nodes (53): RawTracePartText, applyRawTraceRequestLogPolicy(), buildRawTraceConfig(), cleanupStoredRawTraceBundle(), directorySize(), ensureRawTraceDeliveryState(), headerRecordFromUnknown(), isRawTraceSpoolFile() (+45 more)

### Community 31 - "gateway core runtime"
Cohesion: 0.06
Nodes (51): loadPersistedRuntimeState(), replacePersistedRuntimeState(), GatewayNetworkEndpoint, appendGatewayChildOutput(), appendGatewayOutput(), applyCors(), assertGatewayChildRunning(), canLoadGatewayNativeProbe() (+43 more)

### Community 32 - "agents codex app"
Cohesion: 0.07
Nodes (50): bundledCodexCliPath(), codexAppAgentEnv(), codexAppLaunchCommand(), CodexAppLaunchResult, CodexAppLookupResult, codexAppModelCatalogFile(), codexAppSpec, CodexCompatibleAppKind (+42 more)

### Community 33 - "agents local providers"
Cohesion: 0.08
Nodes (48): adoptPeerRotatedGrokAuth(), dateMs(), expiresInMs(), grokAuthFromRecord(), grokBillingEndpoint(), grokBillingMapping, grokBillingResetPaths, grokClientVersion() (+40 more)

### Community 34 - "profiles launch core"
Cohesion: 0.08
Nodes (49): assertAvailableGatewayModels(), ProfileOpenSurface, buildClaudeCodeLaunchPlan(), buildCodexLaunchPlan(), buildGrokLaunchPlan(), buildKiloLaunchPlan(), buildKimiLaunchPlan(), buildOpenCodeLaunchPlan() (+41 more)

### Community 35 - "providers model catalog"
Cohesion: 0.09
Nodes (50): ProviderCatalogModelsResult, addSetValue(), booleanValue(), buildCatalogIndex(), catalogExtraCacheWrite1h(), CatalogIndex, CatalogMatch, catalogModelCanRouteText() (+42 more)

### Community 36 - "agents codex model"
Cohesion: 0.09
Nodes (49): buildCodexModelCatalog(), buildCodexModelCatalogIds(), catalogEntrySupportsImageInput(), CodexCapabilityProfile, codexModelCapabilityProfile(), codexModelCatalogBase64(), CodexModelCatalogItem, codexModelContextWindow() (+41 more)

### Community 37 - "agents zcode profile"
Cohesion: 0.09
Nodes (51): zcodeHomeFromConfigFile(), claudeCodeWrapperCmdScript(), claudeCodeWrapperShellScript(), cmdBotGatewayEnvExports(), cmdCodexlProfileSurfaceExports(), cmdEnvExports(), cmdProfileSurfaceExports(), cmdQuote() (+43 more)

### Community 38 - "platform windows app"
Cohesion: 0.10
Nodes (47): executableFromMacAppBundle(), findInstalledOpenCodeAppExecutable(), findPosixExecutablePid(), findRunningOpenCodeAppPid(), findWindowsExecutablePid(), isDirectory(), isFile(), launchOpenCodeAppProfile() (+39 more)

### Community 39 - "agents bot gateway"
Cohesion: 0.09
Nodes (44): attachBotGatewayStdioErrorHandler(), botGatewayClientRequest(), BotGatewayClientWithRequest, BotGatewayCommand, BotGatewaySdkModule, botGatewayWebSocketTransport(), cancelBotGatewayQrLogin(), closeQrClient() (+36 more)

### Community 40 - "contracts app gateway"
Cohesion: 0.10
Nodes (43): GATEWAY_PLUGIN_PERMISSION_IDS, GATEWAY_PLUGIN_SURFACE_IDS, PluginMarketplaceEntry, assertHttpsUrl(), cachedMarketplaceModulePath(), cloneMarketplaceEntry(), ensureMarketplaceCacheDir(), fetchPluginMarketplace() (+35 more)

### Community 41 - "providers probe codexaccesstokenexpired"
Cohesion: 0.09
Nodes (44): codexAccessTokenExpired(), codexAccountIdFromToken(), codexJwtPayload(), codexMissingRequiredScopes(), codexOauthScope(), codexProbeBackendRequestTransform(), codexRequiredScopes(), codexTokenExpiresAtMs() (+36 more)

### Community 42 - "agents claude app"
Cohesion: 0.09
Nodes (39): ClaudeAppCandidateOptions, claudeAppDesignUrl(), claudeAppLaunchCommand(), ClaudeAppLaunchResult, ClaudeAppLookupResult, claudeCodeModelEnv(), claudeDesignPluginConfig(), claudeElectronArgs() (+31 more)

### Community 43 - "contracts app gatewaypluginappconfig"
Cohesion: 0.06
Nodes (42): GatewayPluginAppConfig, GatewayPluginPermission, GatewayPluginProxyRouteConfig, GatewayPluginSurface, ProviderAccountMeter, ProviderAccountPluginConnectorConfig, ProviderAccountSnapshot, assertJavaScriptModulePath() (+34 more)

### Community 44 - "gateway context archive"
Cohesion: 0.08
Nodes (41): apiKeyMatchesManagedCompactProfile(), apiKeyMatchesProfile(), callHistoryTool(), clampInteger(), ContextArchiveAskOutput, contextArchiveClaudeCodeToolName(), contextArchiveConfigForProfile(), contextArchiveEnabled() (+33 more)

### Community 45 - "gateway core runtime"
Cohesion: 0.09
Nodes (38): authenticateClaudeCode(), authenticateWithBearer(), createGatewayPlugin(), deleteHeader(), grokSupportedResponsesToolTypes, grokToolChoiceNamesRemovedTool(), grokUnsupportedResponsesOptions, HeaderRecord (+30 more)

### Community 46 - "providers account service"
Cohesion: 0.08
Nodes (41): absoluteAccountEndpoint(), cache, CacheEntry, codexOauthCache, CodexOauthRefreshResult, codexOauthRequiredScopes, ConnectorResult, escapeJsonPathSegment() (+33 more)

### Community 47 - "profiles launch service"
Cohesion: 0.11
Nodes (40): launchCodexAppProfile(), launchCodexCompatibleAppProfile(), launchZcodeAppProfile(), refreshCodexCompatibleAppProfileFiles(), shouldEnableCodexMediaPreviewBridge(), openCodeAppLaunchSignature(), activateProfileAppWindow(), cleanupExitedProfileApps() (+32 more)

### Community 48 - "observability route trace"
Cohesion: 0.09
Nodes (30): RequestRouteTrace, RequestRouteTraceDecision, RequestRouteTraceHop, RequestRouteTraceOutcome, RequestRouteTracePhase, RequestRouteTraceTarget, createRequestLogRuntime(), sanitizeHeaders() (+22 more)

### Community 49 - "agents kilo profile"
Cohesion: 0.11
Nodes (37): backupFilePath(), chmodPrivateConfigArtifacts(), chmodPrivateFile(), defaultClientModel(), ensureOriginalSnapshot(), gatewayEndpoint(), isManagedKiloConfigContent(), isRecord() (+29 more)

### Community 50 - "gateway application gateway"
Cohesion: 0.10
Nodes (27): GatewayStatus, hasAvailableGatewayModels(), RouteScriptTestRequest, RouteScriptTestResult, RouteScriptValidationRequest, RouteScriptValidationResult, assertManagedGatewayStartupContinues(), GatewayService (+19 more)

### Community 51 - "models pricing service"
Cohesion: 0.11
Nodes (38): ProviderModelPricing, buildPriceIndex(), divideByMillion(), estimateUsageCostFromCustomPricing(), estimateUsageCostFromIndex(), estimateUsageCostFromPrice(), estimateUsageCostUsd(), estimateUsageCostUsdFromLoadedCatalog() (+30 more)

### Community 52 - "observability request log"
Cohesion: 0.08
Nodes (38): batchNeedsUsagePricing(), buildLogWhereClause(), contentTypeLooksSse(), contentTypeLooksStreaming(), createSseErrorDetector(), detectSseError(), extractUsageFromBillingHeaders(), extractUsageFromBody() (+30 more)

### Community 53 - "agents local providers"
Cohesion: 0.11
Nodes (35): claudeCodeAccountMapping, claudeCodeCandidate(), claudeCodeExpectedKeychainServices(), claudeCodeKeychainAccount(), ClaudeCodeKeychainCandidate, ClaudeCodeLoginScan, claudeCodeProviderAccountConfig(), claudeCodeScanDiagnostic() (+27 more)

### Community 54 - "media service mediaservice"
Cohesion: 0.13
Nodes (15): MediaToolsConfig, callTool(), MediaOperation, MediaRequest, PublicMediaJob, GatewayMediaTransport, createCompletion(), isProviderApiJob() (+7 more)

### Community 55 - "gateway context archive"
Cohesion: 0.15
Nodes (37): appendArchiveFooterToResponse(), appendFooterToJson(), appendFooterToSse(), appendTask(), archiveProtocolError(), archiveResponseRequiresTool(), assertAppendableTurn(), cloneJsonObject() (+29 more)

### Community 56 - "mcp fusion config"
Cohesion: 0.11
Nodes (37): normalizeCoreGatewayVirtualModelProfile(), normalizeUsageVirtualModelProfile(), fusionWebSearchToolCandidates(), BrowserWebSearchMcpIntegration, stringListValue(), browserWebSearchFallbackToolDefinition(), bundledFusionBuiltinMcpEntryPath(), bundledFusionToolFallbackMcpEntryPath() (+29 more)

### Community 57 - "agents claude app"
Cohesion: 0.13
Nodes (33): acquireDirectoryLock(), chmodBestEffort(), ClaudeAppVmStoragePrepareResult, cloneDirectory(), cloneFileWithMacCp(), configuredSeedBundleCandidates(), copyDirectoryContents(), copyFileWithClone() (+25 more)

### Community 58 - "config config repository"
Cohesion: 0.08
Nodes (36): activeGlobalProfile(), createApiKeyConfig(), createGeneratedGatewayApiKey(), endpointPort(), enqueueAppConfigWrite(), ensureGatewayApiKeys(), formatError(), isDefaultSeedApiKey() (+28 more)

### Community 59 - "test unit config"
Cohesion: 0.09
Nodes (30): CCR_EXTENSIONS_PLUGIN_IDS, claudeDesignRuntimePluginConfig(), claudeProductRuntimePluginConfig(), claudeShipRuntimePluginConfig(), isClaudeShipApp(), isDesktopBundledClaudeRuntimePlugin(), isLegacyClaudeDesignAppUrl(), isLegacyClaudeDesignModule() (+22 more)

### Community 60 - "config config repository"
Cohesion: 0.12
Nodes (35): archiveAndRemoveFiles(), collectFileBackups(), ConfigRepositoryOptions, deleteLegacyCleanupRow(), drainLegacyCleanup(), formatError(), insertLegacyBackups(), insertRawApiKeyRows() (+27 more)

### Community 61 - "test unit gateway"
Cohesion: 0.07
Nodes (22): deletePersistedRuntimeState(), createDefaultAppConfig(), testConfig(), testConfig(), baseConfig(), configWithPlugin(), restoreEnv(), withDesktopRuntime() (+14 more)

### Community 62 - "routing route script"
Cohesion: 0.10
Nodes (19): ROUTER_SCRIPT_API_VERSION, RouterRule, RouterRuleScript, RouteScriptInput, basicScriptError(), CircuitState, formatError(), PendingRequest (+11 more)

### Community 63 - "routing route script"
Cohesion: 0.11
Nodes (34): RouterRuleRewriteOperation, applyBodyRewrite(), applyCompiledRouteRewrite(), arrayElementMatches(), cloneJsonValue(), comparableText(), compileRouteRewrite(), compileScriptRouteRewrite() (+26 more)

### Community 64 - "gateway remote control"
Cohesion: 0.15
Nodes (19): CcrRemoteControlRequestContext, CcrRemoteControlService, normalizeEventInputs(), readDirection(), readHeader(), readRecord(), readString(), remoteAfterSeq() (+11 more)

### Community 65 - "media executors gatewaymediaexecutor"
Cohesion: 0.15
Nodes (26): artifactUrlNotAllowedError(), assertArtifactUrlAllowed(), coreGatewayErrorMessage(), effectivePort(), fetchMediaArtifact(), formatGatewayAttempt(), GatewayMediaExecutor, GatewayMediaTarget (+18 more)

### Community 66 - "mcp tool discovery"
Cohesion: 0.12
Nodes (32): GatewayMcpRemoteServerConfig, GatewayMcpStdioServerConfig, GatewayMcpToolInfo, consumeSseEvents(), contentLengthHeaderDelimiter(), createStdioMessageReader(), findExpectedMessage(), hasHeader() (+24 more)

### Community 67 - "proxy service proxyservice"
Cohesion: 0.11
Nodes (17): ProxyNetworkSnapshot, ProxyStatus, proxyCaCertFile(), cloneUpstreamProxy(), closeConnectSocket(), closeServer(), configuredCustomUpstreamProxy(), createProxyStatus() (+9 more)

### Community 68 - "providers account service"
Cohesion: 0.13
Nodes (34): codexAccessTokenExpired(), codexJwtPayload(), codexMissingRequiredScopes(), codexOauthScope(), codexTokenClaims(), codexTokenExpiresAtMs(), codexTokenScopes(), inferMeterKind() (+26 more)

### Community 69 - "contracts app usagecomparisonrow"
Cohesion: 0.09
Nodes (30): UsageComparisonRow, UsageSeriesPoint, buildBuckets(), buildSeries(), buildTotals(), emptySnapshot(), emptyTotals, floorDay() (+22 more)

### Community 70 - "agents opencode profile"
Cohesion: 0.13
Nodes (29): backupFilePath(), chmodPrivateConfigArtifacts(), chmodPrivateFile(), defaultClientModel(), ensureOriginalSnapshot(), gatewayEndpoint(), isManagedOpenCodeConfigContent(), isRecord() (+21 more)

### Community 71 - "mcp toolhub config"
Cohesion: 0.14
Nodes (29): GatewayMcpServerConfig, GROK_MEDIA_FUSION_TOOL_NAMES, bundledMediaToolsMcpEntryPath(), clientGatewayHost(), firstConfiguredApiKey(), formatHost(), hasGatewayEndpoint(), mediaToolsGatewayEndpoint() (+21 more)

### Community 72 - "observability raw trace"
Cohesion: 0.22
Nodes (8): formatError(), cleanupRawTraceBundle(), moveRawTraceBundleToDeadLetter(), positiveInterval(), rawTraceDeadLetterDirectory(), RawTraceSynchronizer, RawTraceSynchronizerDependencies, shouldRecordRequestLogs()

### Community 73 - "proxy system proxy"
Cohesion: 0.13
Nodes (30): customUpstreamProxyFromConfig(), configuredProxyUrlForRequest(), FetchInitWithDispatcher, fetchWithSystemProxy(), formatError(), formatProxyHost(), formatProxyUrl(), getSystemProxyUrlForProtocol() (+22 more)

### Community 74 - "agents bot gateway"
Cohesion: 0.12
Nodes (30): boolEnv(), botGatewayProfileEnv(), botGatewaySdkEnv(), botGatewayWebSocketTransport(), defaultBotGatewayAuthType(), disabledBotGatewayEnv(), isWebhookRelatedBotGatewayKey(), mergeBotGatewayRuntimeConfig() (+22 more)

### Community 75 - "providers new api"
Cohesion: 0.13
Nodes (28): ProviderAccountHttpJsonConnectorConfig, detectedProviderFromHeaders(), DetectedProviderKind, hasNewApiHeaders(), newApiHeaderNames, newApiKeyUsageAccountConfig(), newApiKeyUsageEndpoint(), newApiRootBaseUrl() (+20 more)

### Community 76 - "gateway model catalog"
Cohesion: 0.16
Nodes (30): buildModelCatalogIndex(), fallbackModelCatalogEntry(), findModelCatalogEntry(), gpt56DisplayName(), isRecord(), loadModelCatalogIndex(), ModelCatalogCapabilities, modelCatalogEntryKeys() (+22 more)

### Community 77 - "media service mediaservice"
Cohesion: 0.14
Nodes (25): GatewayMediaProtocol, createGrokMediaModelOptions(), defaultGrokMediaModelSelector(), GrokMediaKind, grokMediaModelKind(), GrokMediaModelOption, grokMediaModelsForProvider(), isImportedGrokAgentProvider() (+17 more)

### Community 78 - "mcp toolhub mcp"
Cohesion: 0.15
Nodes (12): catalogHasChromeLoginImportTool(), env(), envNumber(), expandToolBundleWithCompanionTools(), getDeterministicTaskTools(), isBrowserAutomationTool(), normalizeResolveTaskKey(), readBackendServers() (+4 more)

### Community 79 - "observability request log"
Cohesion: 0.10
Nodes (28): approximateDecodedBytes(), base64EncodingMarker, base64Marker, Base64Range, compactBase64ImagePayloads(), containsBefore(), dataImagePrefix, findBase64ImageRanges() (+20 more)

### Community 80 - "plugins backend service"
Cohesion: 0.11
Nodes (20): BackendService, closeServer(), formatError(), formatHost(), HttpBackendRegistration, isSqliteOpenCorruptionError(), listen(), MaybePromise (+12 more)

### Community 81 - "profiles service buildcodexconfigtoml"
Cohesion: 0.14
Nodes (30): buildCodexConfigToml(), buildCodexContextArchiveMcpBlock(), buildCodexToolHubMcpBlock(), buildKimiProfileConfigToml(), buildSeparateCodexProfileToml(), ensureTrailingNewline(), escapeRegExp(), firstTomlTableIndex() (+22 more)

### Community 82 - "providers account service"
Cohesion: 0.10
Nodes (29): activeProviderCredentials(), codexResetProvider(), effectiveProviderAccount(), effectiveProviderAccountConfig(), effectiveProviderCredentialAccount(), getProviderAccountSnapshots(), hashSensitiveValue(), invalidateProviderAccountSnapshotCache() (+21 more)

### Community 83 - "agents zcode profile"
Cohesion: 0.15
Nodes (25): backupFilePath(), buildZcodeGatewayConfig(), buildZcodeV2Config(), buildZcodeV2ModelCache(), defaultClientModel(), ensureOriginalSnapshot(), gatewayEndpoint(), isLegacyZcodeTomlConfigFile() (+17 more)

### Community 84 - "mcp toolhub mcp"
Cohesion: 0.15
Nodes (5): HttpMcpClient, McpClient, normalizeToolList(), parseHttpJsonRpcResponse(), StdioMcpClient

### Community 85 - "plugins service gatewaypluginservice"
Cohesion: 0.17
Nodes (6): GatewayPluginConfig, InstalledBrowserApp, GatewayPluginService, pluginRuntimeSurfacesEnabled(), pluginSurfaceEnabled(), providerAccountConnectorKey()

### Community 86 - "mcp media tools"
Cohesion: 0.13
Nodes (25): drainInputBuffer(), env(), formatError(), forwardToolCall(), handleJsonRpcRequest(), inputBuffer, isJsonRpcResponse(), isRecord() (+17 more)

### Community 87 - "providers account service"
Cohesion: 0.17
Nodes (27): fetchJson(), grokSubscriptionBoolean(), grokSubscriptionMessage(), grokSubscriptionMeters(), grokSubscriptionRecords(), grokSubscriptionStatus(), grokSubscriptionString(), jsonPathKeyAlternates() (+19 more)

### Community 88 - "agents claude code"
Cohesion: 0.11
Nodes (26): isClaudeCodeManagedModelEnvKey(), ApplyProfileConfigOptions, backupCurrentConfigFile(), backupFilePath(), backupFiles(), chmodFileIfRequested(), claudeCodeSettingsManagedFieldsChanged(), cleanupClaudeCodeToolHubSettingsFile() (+18 more)

### Community 89 - "config constants proxy"
Cohesion: 0.14
Nodes (23): PROXY_CA_CERT_FILE, ProxyCertificateStatus, CertificateAuthority, createCertificateForHost(), createSerialNumber(), ensureProxyCertificateAuthority(), ensureProxyCertificateDerFile(), fingerprintPem() (+15 more)

### Community 90 - "routing config compiler"
Cohesion: 0.14
Nodes (19): RouterRuleRewrite, ClaudeCodeRouterPlugin, CompiledProfileRoutingConfig, CompiledRouterConfig, CompiledRouterRule, compileProfileRoutings(), compileRouterConfig(), CompileRouterConfigOptions (+11 more)

### Community 91 - "gateway features codex"
Cohesion: 0.15
Nodes (23): codexMultiAgentBridgeEnabled(), codexMultiAgentBridgeModelEligible(), codexMultiAgentFunctionName(), codexMultiAgentFunctionTool(), flushSseTransform(), isCodexUserAgent(), modelNameForMultiAgentBridge(), multiAgentToolNames (+15 more)

### Community 92 - "media service mediaservice"
Cohesion: 0.14
Nodes (19): ImageEditRequest, ImageGenerateRequest, MediaArtifact, MediaExecutionContext, MediaExecutionResult, MediaJobError, MediaJobStatus, MediaUsage (+11 more)

### Community 93 - "agents claude code"
Cohesion: 0.13
Nodes (22): assignModelAliasEnv(), chinaTimeZones, CLAUDE_CODE_MANAGED_MODEL_ENV_KEYS, claudeCodeMcpConfigEnv(), claudeCodeModelEnv(), ClaudeCodeModelSelection, claudeCodeUtcTimezoneEnvOverride(), clearClaudeCodeManagedModelEnv() (+14 more)

### Community 94 - "gateway claude code"
Cohesion: 0.13
Nodes (25): builtInAgentPolicyId(), builtInAgentRouteMatches(), builtInAgentUserAgentNeedle(), ClaudeCodeSubagentToolKind, explicitClientModelCanOverrideBuiltInClaudeCodeRoute(), findMatchedCompiledRule(), isSubagentModelPlaceholder(), mergeConfiguredRouteDecisions() (+17 more)

### Community 95 - "mcp grok media"
Cohesion: 0.12
Nodes (20): formatError(), handleJsonRpcRequest(), handleMediaArtifactRequest(), handleMediaToolsMcpRequest(), isRecord(), JsonPrimitive, jsonRpcError(), JsonRpcRequest (+12 more)

### Community 96 - "config config bundledruntimepluginmodulecandi"
Cohesion: 0.12
Nodes (24): bundledRuntimePluginModuleCandidates(), ccrExtensionsRootCandidates(), ccrExtensionsRootFromLegacyModule(), defaultOverviewWidgetSize(), defaultOverviewWidgetVariant(), inferRouterFallbackMode(), isShareOverviewWidgetType(), overviewWidgetId() (+16 more)

### Community 97 - "mcp toolhub mcp"
Cohesion: 0.11
Nodes (21): buildSearchRefinementFeedback(), buildSearchSystemPrompt(), createEmptyCacheStore(), firstJsonObject(), isRecord(), isStringRecord(), normalizeCacheStore(), normalizeInputSchema() (+13 more)

### Community 98 - "observability request log"
Cohesion: 0.14
Nodes (18): preloadUsagePriceCatalog(), isRecord(), JsonRecord, modelFromPayload(), normalizeModel(), parseJsonBody(), requestLogRequestedModel(), requestLogResponseModel() (+10 more)

### Community 99 - "usage billing sync"
Cohesion: 0.17
Nodes (17): findProviderByPublicOrInternalName(), parseProviderCredentialInternalName(), resolveResponseProviderProtocol(), authoritativeUsageCost(), billingProviderForProtocol(), finiteNumber(), GatewayBillingSynchronizer, GatewayBillingSynchronizerOptions (+9 more)

### Community 100 - "agents claude app"
Cohesion: 0.17
Nodes (20): ClaudeAppCdpLogger, ClaudeAppDesignCdpOptions, claudeAppDesignFeatureScript(), claudeAppDesignFrameScript(), claudeAppDesktopDesignUrl(), DevToolsTarget, FetchRequestPausedParams, forceOpenClaudeAppDesignViaCdp() (+12 more)

### Community 101 - "mcp network capture"
Cohesion: 0.14
Nodes (21): ProxyNetworkExchange, callTool(), captureMatchesQuery(), captureStatus(), clampInteger(), clearCaptures(), getCapture(), JsonPrimitive (+13 more)

### Community 102 - "observability request log"
Cohesion: 0.17
Nodes (12): admissionFromRow(), CommittedAdmissionLookup, dateMs(), ensureAdmissionBodyCapturePolicyColumn(), formatError(), nonNegativeInteger(), normalizeBodyCapturePolicy(), normalizeState() (+4 more)

### Community 103 - "agents codex media"
Cohesion: 0.14
Nodes (18): CodexMediaPreviewBridgeOptions, codexMediaPreviewInjectionScript(), CodexMediaPreviewLogger, codexMediaPreviewPageBootstrap(), detectMediaMimeType(), DevToolsTarget, isCodexAppPageTarget(), loadCodexMediaArtifact() (+10 more)

### Community 104 - "agents local providers"
Cohesion: 0.20
Nodes (21): readCodexLocalModelCatalog(), firstString(), isLoopbackUrl(), modelDisplayNamesForModels(), readJsonRecord(), uniqueStrings(), isZcodeModelProvider(), LocalAgentModelCatalog (+13 more)

### Community 105 - "config constants provider"
Cohesion: 0.19
Nodes (21): PROVIDER_ICON_CACHE_DIR, ProviderIconDetectionRequest, ProviderIconDetectionResult, detectProviderIcon(), discoverHtmlProviderIconUrls(), downloadProviderIconCandidate(), fetchWithTimeout(), findCachedProviderIcon() (+13 more)

### Community 106 - "mcp fusion tool"
Cohesion: 0.15
Nodes (21): callTool(), drainInputBuffer(), env(), formatError(), handleJsonRpcRequest(), inputBuffer, isRecord(), JsonPrimitive (+13 more)

### Community 107 - "routing route script"
Cohesion: 0.19
Nodes (21): assertHttpUrl(), compileScript(), controlledFetch(), createRouteScriptBridge(), evaluateRequest(), failureResponse(), formatError(), isRecord() (+13 more)

### Community 108 - "agents bot gateway"
Cohesion: 0.21
Nodes (20): bluetoothFallbackLabel(), bluetoothScanTargetFromObject(), collectBluetoothScanTargets(), collectBluetoothScanTargetsFromText(), collectBluetoothTargetsFromCommand(), commandStdout(), execFileAsync, firstStringField() (+12 more)

### Community 109 - "agents codex model"
Cohesion: 0.19
Nodes (21): codexModelCatalogJson(), applyClaudeDesignProfile(), applyCodexProfile(), applyGrokProfile(), applyKimiProfile(), applyPiProfile(), applyProfileConfig(), applyProfileRuntimeConfig() (+13 more)

### Community 110 - "config config repository"
Cohesion: 0.20
Nodes (8): ConfigRepository, configureSqliteDatabase(), createSchema(), replaceApiKeyRows(), replaceAppConfigRow(), replaceJsonRow(), secureDatabaseFilePermissions(), securePathPermissions()

### Community 111 - "config constants legacy"
Cohesion: 0.12
Nodes (17): APP_CONFIG_DB_FILE, CERTDIR, CONTEXT_ARCHIVE_DB_FILE, LEGACY_ACTIVE_CONFIG_FILE, LEGACY_API_KEYS_DB_FILE, LEGACY_CONFIG_FILE, LEGACY_WINDOWS_CONFIG_FILE, LEGACY_WINDOWS_CONFIGDIR (+9 more)

### Community 112 - "benchmark request log"
Cohesion: 0.18
Nodes (17): closeHttpServer(), createRecord(), createRequestBody(), createTarget(), createTrace(), createWebRecord(), createWebTarget(), handleWebRequest() (+9 more)

### Community 113 - "agents pi profile"
Cohesion: 0.21
Nodes (18): chmodPrivateDir(), chmodPrivateFile(), gatewayEndpoint(), homeDir(), piModelConfig(), piModelsJson(), PiProfileConfigWriteResult, piProfileModels() (+10 more)

### Community 114 - "contracts app usagestatsrange"
Cohesion: 0.21
Nodes (18): UsageStatsRange, UsageStatsSnapshot, applyMaxShare(), buildRecentRequestRows(), formatRequestTime(), normalizeCost(), normalizeFilterValue(), normalizeLabel() (+10 more)

### Community 115 - "plugins backend service"
Cohesion: 0.14
Nodes (9): assertSqliteDatabaseIntegrity(), normalizeSqliteParams(), normalizeSqliteRow(), normalizeSqliteValue(), sqlCanReturnRows(), SqlDatabase, SqliteCompatDatabase, SqliteCompatStatement (+1 more)

### Community 116 - "routing route script"
Cohesion: 0.19
Nodes (14): buildRouteScriptInput(), BuildRouteScriptInputOptions, cloneJson(), containsImage(), isRecord(), lastUserText(), readHeader(), requestHeaders() (+6 more)

### Community 117 - "usage store asnumber"
Cohesion: 0.15
Nodes (18): asNumber(), asString(), extractUsageFromBillingHeaders(), extractUsageFromBody(), extractUsageSnapshot(), hasUsageNumbers(), mergeUsageSnapshots(), normalizeCount() (+10 more)

### Community 118 - "test integration mcp"
Cohesion: 0.15
Nodes (12): availablePort(), close(), listen(), materializeProviderPlugins(), mcpEndpoint(), mcpRequest(), parseToolResult(), replacePlaceholders() (+4 more)

### Community 119 - "observability request log"
Cohesion: 0.18
Nodes (17): RAW_TRACE_SPOOL_DIR, RequestLogRawTraceFile, chain, configuration, errorCode(), formatError(), isCompleteJsonContainer(), readRawTraceBody() (+9 more)

### Community 120 - "mcp toolhub mcp"
Cohesion: 0.17
Nodes (10): cloneServerConfig(), hashToolHubMcpServerConfig(), matchesUniqueSuffix(), normalizeTargetServerNames(), resolveCatalogItemName(), serverNamespaces(), stableJsonStringify(), toIdentifier() (+2 more)

### Community 121 - "routing policy engine"
Cohesion: 0.16
Nodes (10): AgentRequestEnricher, applyAgentRequestEnrichers(), RoutePolicy, RoutePolicyEngine, RoutePolicyMatch, adaptRouteRequestBody(), restoreRouteRequestBody(), rewriteRouteModelInUrl() (+2 more)

### Community 122 - "profiles api key"
Cohesion: 0.19
Nodes (15): replacePersistedApiKeys(), ProfileConfig, generateProfileApiKey(), profileApiKeyId(), profileApiKeyName(), ProfileApiKeySource, ProfileApiKeySyncOptions, ProfileApiKeySyncResult (+7 more)

### Community 123 - "gateway core runtime"
Cohesion: 0.18
Nodes (15): basePath(), ccrAuthHeaderNames, ccrRoutingHeaderNames, clientAuthHeaderNames, createGatewayPlugin(), headerValues(), joinUrlPath(), mergeUpstreamProviderHeaders() (+7 more)

### Community 124 - "media storage mediaartifactstore"
Cohesion: 0.21
Nodes (10): localImageDataUrl(), detectMediaBufferType(), detectMediaType(), extensionForMimeType(), fileNameWithExtension(), hashFile(), isPathInside(), JobStoreFile (+2 more)

### Community 125 - "web management server"
Cohesion: 0.17
Nodes (17): inspectPluginDirectory(), isAllPluginPermissionsKey(), isAllPluginSurfacesKey(), isFile(), normalizePluginPermission(), normalizePluginSurface(), parsePluginPermissions(), parsePluginSurfaces() (+9 more)

### Community 126 - "test integration gateway"
Cohesion: 0.15
Nodes (8): codexMediaPreviewBridgeForTest, prepareCodexAppCdpUserDataDir(), sleep(), waitFor(), waitForTcpListener(), mp4, png, token

### Community 127 - "routing execution plan"
Cohesion: 0.17
Nodes (14): RouterFallbackConfig, GatewayModelRef, ProviderModelRef, RouteAttemptPlan, RouteDecision, RouteDiagnosticCode, RouteExecutionPlan, RouteRequest (+6 more)

### Community 128 - "providers account service"
Cohesion: 0.15
Nodes (16): jsonPathFilterMatches(), kimiCodeUsageLimitLabel(), kimiCodeUsageMeter(), kimiCodeUsageResetAt(), mergeConnectorResults(), meterRemainingRatio(), mostSevereStatus(), nextJsonPathBoundary() (+8 more)

### Community 129 - "usage store usagestore"
Cohesion: 0.17
Nodes (8): UsageStatsFilter, UsageTotals, cleanupSqliteTempCopy(), configureSqliteDatabase(), copySqliteDatabaseToTemp(), ensureUsageSchema(), sqlString(), UsageStore

### Community 130 - "platform socket compat"
Cohesion: 0.22
Nodes (11): CoreServerOptions, parseCoreServerArgs(), parsePort(), printHelp(), requiredArg(), runCoreServer(), installSocketTypeOfServiceCompat(), isIgnorableSocketTypeOfServiceError() (+3 more)

### Community 131 - "mcp browser web"
Cohesion: 0.25
Nodes (13): drainFrames(), formatError(), forwardPayload(), isJsonRpcId(), isRecord(), jsonRpcError(), readContentLength(), readJsonRpcId() (+5 more)

### Community 132 - "media storage mediajobstore"
Cohesion: 0.21
Nodes (4): MediaJob, safeTokenEqual(), formatError(), MediaJobStore

### Community 133 - "package scripts test"
Cohesion: 0.14
Nodes (13): bin, ccr-core-server, description, engines, node, main, name, private (+5 more)

### Community 135 - "test unit providers"
Cohesion: 0.19
Nodes (12): GatewayProviderProbeResult, newApiKeyUsageFallbackMessageForTest(), newApiKeyUsageMetersForTest(), newApiUserSelfMetersForTest(), checkGatewayProviderConnectivity(), probeGatewayProvider(), probeGatewayProviderCandidates(), providerProbeCacheTtl() (+4 more)

### Community 136 - "proxy service proxyservice"
Cohesion: 0.19
Nodes (11): ProxyCertificateInstallResult, execFilePromise(), macosManualCertificateInstallCommand(), macosSystemCertificateInstallScript(), macosTerminalCertificateInstallScript(), openMacosTerminalCertificateInstaller(), quoteAppleScriptString(), quoteShellArg() (+3 more)

### Community 137 - "gateway http request"
Cohesion: 0.26
Nodes (4): isContextArchiveMcpPath(), GatewayHttpRequestHandler, GatewayHttpRequestHandlerDependencies, isNetworkCaptureMcpPath()

### Community 139 - "mcp toolhub mcp"
Cohesion: 0.24
Nodes (3): consumeSseEvents(), SseMcpClient, toError()

### Community 140 - "runtime app paths"
Cohesion: 0.26
Nodes (12): fallbackAppDataDir(), fallbackUserDataDir(), readConfiguredPath(), resolveRuntimeAppPath(), resolveRuntimeConfigDir(), resolveRuntimeDataDir(), RuntimeAppPaths, RuntimePathName (+4 more)

### Community 141 - "package dependencies the"
Cohesion: 0.15
Nodes (13): better-sqlite3, node-forge, dependencies, better-sqlite3, node-forge, pm2, @the-next-ai/ai-gateway, @the-next-ai/bot-gateway-sdk (+5 more)

### Community 142 - "test unit agents"
Cohesion: 0.19
Nodes (6): finalizeContextArchiveRequest(), allStrings(), mockReplay(), ready(), responseResult(), testConfig()

### Community 143 - "test integration mcp"
Cohesion: 0.21
Nodes (5): createDelayedResolverServer(), createMcpHttpServer(), createTaskAwareResolverServer(), readJsonBody(), readResolverQuery()

### Community 144 - "gateway context archive"
Cohesion: 0.29
Nodes (11): ArchiveRetention, ArchiveRoute, ArchiveSnapshot, ArchiveSnapshotStatus, optionalNumber(), parseOptionalRoute(), parseRecord(), readString() (+3 more)

### Community 145 - "gateway core runtime"
Cohesion: 0.26
Nodes (10): GatewayStartMessage, installVirtualConfigFile(), isManagedConfigPath(), isObject(), isVirtualConfigPath(), MutableFs, parseStartMessage(), readsText() (+2 more)

### Community 146 - "gateway context archive"
Cohesion: 0.36
Nodes (4): ContextArchiveConfig, constantTimeEqual(), ContextArchiveService, isInsufficientArchiveAnswer()

### Community 147 - "plugins service gatewaypluginservice"
Cohesion: 0.27
Nodes (8): RegisteredHttpBackend, createPluginLogger(), GatewayPluginHttpBackendRegistration, GatewayPluginRouteRegistration, pluginPermissionList(), readBody(), readJson(), sendJson()

### Community 148 - "test unit agents"
Cohesion: 0.31
Nodes (8): fakeSecurityBody(), restoreEnv(), withClaudeCodeHome(), withEnv(), withFakeSecurityFailure(), withFakeSecurityKeychain(), withFakeSecurityOutput(), withFakeSecurityScript()

### Community 149 - "config config assertprovideraccountapikeytarg"
Cohesion: 0.24
Nodes (10): assertProviderAccountApiKeyTargetsAreSafe(), assertProviderApiKeysAreSafe(), assertProviderCredentialAccountApiKeyTargetsAreSafe(), providerAccountConnectorApiKeyEndpoints(), providerApiKey(), providerBaseUrl(), providerCredentialApiKey(), providerRuntimeIdCandidate() (+2 more)

### Community 150 - "src contracts i18n"
Cohesion: 0.29
Nodes (9): browserErrorI18nLanguage(), ErrorI18nLanguage, formatLocalizedErrorMessage(), PatternTranslator, readBrowserLanguagePreference(), resolveErrorI18nLanguage(), translateErrorMessage(), zhExactErrorMessages (+1 more)

### Community 151 - "mcp toolhub mcp"
Cohesion: 0.29
Nodes (10): discardLeadingNewlines(), drainInputBuffer(), formatError(), handleJsonRpcMessage(), handleJsonRpcRequest(), jsonRpcError(), jsonRpcResult(), metaTools() (+2 more)

### Community 152 - "test unit providers"
Cohesion: 0.24
Nodes (7): localAgentProviderAccountCredentialForTest(), localCodexAccountCredentialForTest(), setProviderAccountWebContentFetchHandler(), base64url(), jwt(), useTemporaryCodexHome(), useTemporaryHome()

### Community 153 - "test integration agents"
Cohesion: 0.33
Nodes (5): codexCliMiddlewareRuntimeScript(), evaluateRuntimeFunction(), extractRuntimeFunctionSource(), waitForJsonLines(), writeRuntimeScript()

### Community 154 - "agents zcode model"
Cohesion: 0.31
Nodes (8): CodexModelCatalog, buildZcodeModelCatalog(), ZcodeModelCatalogConfig, zcodeModelCatalogEntry(), zcodeModelCatalogJson(), ZcodeModelResolutionConfig, claudeCodeEffectiveMaxInputTokens(), modelCatalogMaxInputTokens()

### Community 155 - "mcp network capture"
Cohesion: 0.28
Nodes (9): appVersion(), formatError(), handleJsonRpcRequest(), handleNetworkCaptureMcpRequest(), isRecord(), jsonRpcError(), jsonRpcResult(), readRequestBody() (+1 more)

### Community 156 - "plugins service gatewaypluginservice"
Cohesion: 0.33
Nodes (7): buildPluginProxyUpstreamUrl(), joinUrlPaths(), matchesHost(), matchesPathPrefix(), matchProxyRoute(), normalizeRoutePath(), resolveStripPathPrefix()

### Community 157 - "plugins service gatewaypluginservice"
Cohesion: 0.33
Nodes (5): enabledPluginIds(), formatError(), isDesktopOnlyClaudeBrowserPlugin(), pluginAvailableInCurrentRuntime(), stopReasonForPlugin()

### Community 158 - "profiles service clearglobalprofiletakeoverma"
Cohesion: 0.36
Nodes (9): clearGlobalProfileTakeoverMarker(), dedupeGlobalProfileTakeovers(), globalProfileTakeoverRecords(), readGlobalProfileTakeoverMarker(), restoreGlobalProfileConfigsOnExit(), restoreGlobalProfileTakeoverRecords(), storeGlobalProfileTakeoverRecords(), synchronizeGlobalProfileTakeovers() (+1 more)

### Community 159 - "providers account service"
Cohesion: 0.36
Nodes (9): connectorError(), connectorId(), connectorSource(), formatError(), providerAccountConnectorUsesProviderApiKey(), readConnectorType(), resolveConnector(), resolvePluginConnector() (+1 more)

### Community 160 - "src usage normalization"
Cohesion: 0.43
Nodes (6): inputIncludesCacheTokens(), inputIncludesCacheTokensForPath(), inputIncludesCacheTokensForProtocol(), normalizeCount(), normalizeUsageInputTokens(), UsageTokenAccounting

### Community 161 - "test integration mcp"
Cohesion: 0.29
Nodes (3): readJsonRpcFrame(), sendJsonRpc(), waitFor()

### Community 162 - "test unit config"
Cohesion: 0.36
Nodes (4): readBackupCount(), readCleanupCount(), readMigration(), withReadonlyDatabase()

### Community 163 - "ref"
Cohesion: 0.25
Nodes (7): compilerOptions, rootDir, extends, include, src/**/*.d.ts, src/**/*.ts, ../../tsconfig.json

### Community 164 - "config onboarding state"
Cohesion: 0.43
Nodes (6): loadPersistedAppSetting(), replacePersistedAppSetting(), ONBOARDING_FINISHED_FILE, formatError(), loadOnboardingFinished(), markOnboardingFinished()

### Community 165 - "mcp toolhub mcp"
Cohesion: 0.38
Nodes (6): buildExecutionPlanJs(), buildLocalFallbackWorkflowSketch(), buildSequentialExecutionPlanJs(), getLocalFallbackPreferredTools(), scoreLocalCatalogMatch(), tokenizeLocalSearchText()

### Community 166 - "models catalog file"
Cohesion: 0.57
Nodes (5): LoadedModelCatalogPayload, loadModelCatalogPayload(), modelCatalogPathCandidates(), resolveModelCatalogPath(), uniqueStrings()

### Community 167 - "benchmark request log"
Cohesion: 0.53
Nodes (4): main(), parseArgs(), percentile(), positiveInteger()

### Community 168 - "agents cdp client"
Cohesion: 0.33
Nodes (4): CdpClientOptions, CdpError, CdpMessage, webSocketDataToString()

### Community 169 - "routing failure classifier"
Cohesion: 0.40
Nodes (5): RouterFallbackMode, classifyRouteFailure(), classifyStatus(), RouteFailureClass, RouteFailureDecision

### Community 170 - "mcp toolhub mcp"
Cohesion: 0.53
Nodes (3): buildToolReference(), ToolReferenceAnalyzer, uniqueToolReferences()

### Community 171 - "test unit config"
Cohesion: 0.47
Nodes (3): readBackupCount(), readCleanupCount(), withReadonlyDatabase()

### Community 172 - "gateway runtime change"
Cohesion: 0.50
Nodes (3): mediaToolsConfigFromRawForTest(), virtualModelProfileFromRawForTest(), shouldRestartGatewayForRuntimeConfigChange()

### Community 173 - "providers account service"
Cohesion: 0.50
Nodes (5): jsonErrorMessage(), readableResponseSnippet(), readJsonResponse(), responseLooksJson(), tokenRefreshErrorMessage()

### Community 175 - "usage store buildusagewhereclause"
Cohesion: 0.67
Nodes (4): buildUsageWhereClause(), isRecord(), normalizeUsageFilter(), normalizeUsageQueryOptions()

## Knowledge Gaps
- **673 isolated node(s):** `iterations`, `parsePasses`, `payload`, `legacyMs`, `optimizedMs` (+668 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppConfig` connect `gateway internal shared` to `contracts app default`, `gateway features hosted`, `gateway http io`, `web management server`, `gateway features hosted`, `providers runtime topology`, `profiles launch service`, `agents claude app`, `src config config`, `gateway claude code`, `gateway features context`, `agents zcode profile`, `gateway upstream executor`, `agents zcode model`, `proxy service proxyservice`, `gateway features model`, `observability raw trace`, `gateway core runtime`, `agents codex app`, `plugins service gatewaypluginservice`, `profiles launch core`, `agents codex model`, `agents claude app`, `contracts app gatewaypluginappconfig`, `gateway context archive`, `gateway runtime change`, `providers account service`, `agents kilo profile`, `gateway application gateway`, `models pricing service`, `media service mediaservice`, `mcp fusion config`, `proxy service proxyservice`, `contracts app usagecomparisonrow`, `agents opencode profile`, `mcp toolhub config`, `proxy system proxy`, `agents bot gateway`, `media service mediaservice`, `agents zcode profile`, `plugins service gatewaypluginservice`, `routing config compiler`, `gateway features codex`, `media service mediaservice`, `usage billing sync`, `agents pi profile`?**
  _High betweenness centrality (0.084) - this node is a cross-community bridge._
- **Why does `delay()` connect `mcp toolhub mcp` to `routing config compiler`?**
  _High betweenness centrality (0.038) - this node is a cross-community bridge._
- **Why does `MediaService` connect `media service mediaservice` to `media storage mediajobstore`, `src config config`, `media service mediaservice`, `media service mediaservice`, `gateway application gateway`, `gateway internal shared`, `test integration mcp`, `media storage mediaartifactstore`, `mcp grok media`?**
  _High betweenness centrality (0.030) - this node is a cross-community bridge._
- **What connects `iterations`, `parsePasses`, `payload` to the rest of the system?**
  _673 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `contracts app default` be split into smaller, more focused modules?**
  _Cohesion score 0.01512747275997038 - nodes in this community are weakly interconnected._
- **Should `gateway features hosted` be split into smaller, more focused modules?**
  _Cohesion score 0.05327263779527559 - nodes in this community are weakly interconnected._
- **Should `observability request log` be split into smaller, more focused modules?**
  _Cohesion score 0.033129459734964326 - nodes in this community are weakly interconnected._