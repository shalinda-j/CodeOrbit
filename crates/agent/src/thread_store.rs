﻿use std::cell::{Ref, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use agent_settings::{AgentProfile, AgentProfileId, AgentSettings, CompletionMode};
use anyhow::{Context as _, Result, anyhow};
use assistant_tool::{ToolId, ToolSource, ToolWorkingSet};
use chrono::{DateTime, Utc};
use collections::HashMap;
use context_server::ContextServerId;
use futures::channel::{mpsc, oneshot};
use futures::future::{self, BoxFuture, Shared};
use futures::{FutureExt as _, StreamExt as _};
use gpui::{
    App, BackgroundExecutor, Context, Entity, EventEmitter, Global, ReadGlobal, SharedString,
    Subscription, Task, prelude::*,
};

use language_model::{LanguageModelToolResultContent, LanguageModelToolUseId, Role, TokenUsage};
use project::context_server_store::{ContextServerStatus, ContextServerStore};
use project::{Project, ProjectItem, ProjectPath, Worktree};
use prompt_store::{
    ProjectContext, PromptBuilder, PromptId, PromptStore, PromptsUpdatedEvent, RulesFileContext,
    UserRulesContext, WorktreeContext,
};
use serde::{Deserialize, Serialize};
use settings::{Settings as _, SettingsStore};
use ui::Window;
use util::ResultExt as _;

use crate::context_server_tool::ContextServerTool;
use crate::thread::{
    DetailedSummaryState, ExceededWindowError, MessageId, ProjectSnapshot, Thread, ThreadId,
};
use indoc::indoc;
use sqlez::{
    bindable::{Bind, Column},
    connection::Connection,
    statement::Statement,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "zstd")]
    Zstd,
}

impl Bind for DataType {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        let value = match self {
            DataType::Json => "json",
            DataType::Zstd => "zstd",
        };
        value.bind(statement, start_index)
    }
}

impl Column for DataType {
    fn column(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)> {
        let (value, next_index) = String::column(statement, start_index)?;
        let data_type = match value.as_str() {
            "json" => DataType::Json,
            "zstd" => DataType::Zstd,
            _ => anyhow::bail!("Unknown data type: {}", value),
        };
        Ok((data_type, next_index))
    }
}

const RULES_FILE_NAMES: [&'static str; 8] = [
    ".rules",
    ".cursorrules",
    ".windsurfrules",
    ".clinerules",
    ".github/copilot-instructions.md",
    "CLAUDE.md",
    "AGENT.md",
    "AGENTS.md",
];

pub fn init(cx: &mut App) {
    ThreadsDatabase::init(cx);
}

/// A system prompt shared by all threads created by this ThreadStore
#[derive(Clone, Default)]
pub struct SharedProjectContext(Rc<RefCell<Option<ProjectContext>>>);

impl SharedProjectContext {
    pub fn borrow(&self) -> Ref<Option<ProjectContext>> {
        self.0.borrow()
    }
}

pub type TextThreadStore = assistant_context_editor::ContextStore;

pub struct ThreadStore {
    project: Entity<Project>,
    tools: Entity<ToolWorkingSet>,
    prompt_builder: Arc<PromptBuilder>,
    prompt_store: Option<Entity<PromptStore>>,
    context_server_tool_ids: HashMap<ContextServerId, Vec<ToolId>>,
    threads: Vec<SerialiCodeOrbitThreadMetadata>,
    project_context: SharedProjectContext,
    reload_system_prompt_tx: mpsc::Sender<()>,
    _reload_system_prompt_task: Task<()>,
    _subscriptions: Vec<Subscription>,
}

pub struct RulesLoadingError {
    pub message: SharedString,
}

impl EventEmitter<RulesLoadingError> for ThreadStore {}

impl ThreadStore {
    pub fn load(
        project: Entity<Project>,
        tools: Entity<ToolWorkingSet>,
        prompt_store: Option<Entity<PromptStore>>,
        prompt_builder: Arc<PromptBuilder>,
        cx: &mut App,
    ) -> Task<Result<Entity<Self>>> {
        cx.spawn(async move |cx| {
            let (thread_store, ready_rx) = cx.update(|cx| {
                let mut option_ready_rx = None;
                let thread_store = cx.new(|cx| {
                    let (thread_store, ready_rx) =
                        Self::new(project, tools, prompt_builder, prompt_store, cx);
                    option_ready_rx = Some(ready_rx);
                    thread_store
                });
                (thread_store, option_ready_rx.take().unwrap())
            })?;
            ready_rx.await?;
            Ok(thread_store)
        })
    }

    fn new(
        project: Entity<Project>,
        tools: Entity<ToolWorkingSet>,
        prompt_builder: Arc<PromptBuilder>,
        prompt_store: Option<Entity<PromptStore>>,
        cx: &mut Context<Self>,
    ) -> (Self, oneshot::Receiver<()>) {
        let mut subscriptions = vec![
            cx.observe_global::<SettingsStore>(move |this: &mut Self, cx| {
                this.load_default_profile(cx);
            }),
            cx.subscribe(&project, Self::handle_project_event),
        ];

        if let Some(prompt_store) = prompt_store.as_ref() {
            subscriptions.push(cx.subscribe(
                prompt_store,
                |this, _prompt_store, PromptsUpdatedEvent, _cx| {
                    this.enqueue_system_prompt_reload();
                },
            ))
        }

        // This channel and task prevent concurrent and redundant loading of the system prompt.
        let (reload_system_prompt_tx, mut reload_system_prompt_rx) = mpsc::channel(1);
        let (ready_tx, ready_rx) = oneshot::channel();
        let mut ready_tx = Some(ready_tx);
        let reload_system_prompt_task = cx.spawn({
            let prompt_store = prompt_store.clone();
            async move |thread_store, cx| {
                loop {
                    let Some(reload_task) = thread_store
                        .update(cx, |thread_store, cx| {
                            thread_store.reload_system_prompt(prompt_store.clone(), cx)
                        })
                        .ok()
                    else {
                        return;
                    };
                    reload_task.await;
                    if let Some(ready_tx) = ready_tx.take() {
                        ready_tx.send(()).ok();
                    }
                    reload_system_prompt_rx.next().await;
                }
            }
        });

        let this = Self {
            project,
            tools,
            prompt_builder,
            prompt_store,
            context_server_tool_ids: HashMap::default(),
            threads: Vec::new(),
            project_context: SharedProjectContext::default(),
            reload_system_prompt_tx,
            _reload_system_prompt_task: reload_system_prompt_task,
            _subscriptions: subscriptions,
        };
        this.load_default_profile(cx);
        this.register_context_server_handlers(cx);
        this.reload(cx).detach_and_log_err(cx);
        (this, ready_rx)
    }

    fn handle_project_event(
        &mut self,
        _project: Entity<Project>,
        event: &project::Event,
        _cx: &mut Context<Self>,
    ) {
        match event {
            project::Event::WorktreeAdded(_) | project::Event::WorktreeRemoved(_) => {
                self.enqueue_system_prompt_reload();
            }
            project::Event::WorktreeUpdatedEntries(_, items) => {
                if items.iter().any(|(path, _, _)| {
                    RULES_FILE_NAMES
                        .iter()
                        .any(|name| path.as_ref() == Path::new(name))
                }) {
                    self.enqueue_system_prompt_reload();
                }
            }
            _ => {}
        }
    }

    fn enqueue_system_prompt_reload(&mut self) {
        self.reload_system_prompt_tx.try_send(()).ok();
    }

    // Note that this should only be called from `reload_system_prompt_task`.
    fn reload_system_prompt(
        &self,
        prompt_store: Option<Entity<PromptStore>>,
        cx: &mut Context<Self>,
    ) -> Task<()> {
        let worktrees = self
            .project
            .read(cx)
            .visible_worktrees(cx)
            .collect::<Vec<_>>();
        let worktree_tasks = worktrees
            .into_iter()
            .map(|worktree| {
                Self::load_worktree_info_for_system_prompt(worktree, self.project.clone(), cx)
            })
            .collect::<Vec<_>>();
        let default_user_rules_task = match prompt_store {
            None => Task::ready(vec![]),
            Some(prompt_store) => prompt_store.read_with(cx, |prompt_store, cx| {
                let prompts = prompt_store.default_prompt_metadata();
                let load_tasks = prompts.into_iter().map(|prompt_metadata| {
                    let contents = prompt_store.load(prompt_metadata.id, cx);
                    async move { (contents.await, prompt_metadata) }
                });
                cx.background_spawn(future::join_all(load_tasks))
            }),
        };

        cx.spawn(async move |this, cx| {
            let (worktrees, default_user_rules) =
                future::join(future::join_all(worktree_tasks), default_user_rules_task).await;

            let worktrees = worktrees
                .into_iter()
                .map(|(worktree, rules_error)| {
                    if let Some(rules_error) = rules_error {
                        this.update(cx, |_, cx| cx.emit(rules_error)).ok();
                    }
                    worktree
                })
                .collect::<Vec<_>>();

            let default_user_rules = default_user_rules
                .into_iter()
                .flat_map(|(contents, prompt_metadata)| match contents {
                    Ok(contents) => Some(UserRulesContext {
                        uuid: match prompt_metadata.id {
                            PromptId::User { uuid } => uuid,
                            PromptId::EditWorkflow => return None,
                        },
                        title: prompt_metadata.title.map(|title| title.to_string()),
                        contents,
                    }),
                    Err(err) => {
                        this.update(cx, |_, cx| {
                            cx.emit(RulesLoadingError {
                                message: format!("{err:?}").into(),
                            });
                        })
                        .ok();
                        None
                    }
                })
                .collect::<Vec<_>>();

            this.update(cx, |this, _cx| {
                *this.project_context.0.borrow_mut() =
                    Some(ProjectContext::new(worktrees, default_user_rules));
            })
            .ok();
        })
    }

    fn load_worktree_info_for_system_prompt(
        worktree: Entity<Worktree>,
        project: Entity<Project>,
        cx: &mut App,
    ) -> Task<(WorktreeContext, Option<RulesLoadingError>)> {
        let root_name = worktree.read(cx).root_name().into();

        let rules_task = Self::load_worktree_rules_file(worktree, project, cx);
        let Some(rules_task) = rules_task else {
            return Task::ready((
                WorktreeContext {
                    root_name,
                    rules_file: None,
                },
                None,
            ));
        };

        cx.spawn(async move |_| {
            let (rules_file, rules_file_error) = match rules_task.await {
                Ok(rules_file) => (Some(rules_file), None),
                Err(err) => (
                    None,
                    Some(RulesLoadingError {
                        message: format!("{err}").into(),
                    }),
                ),
            };
            let worktree_info = WorktreeContext {
                root_name,
                rules_file,
            };
            (worktree_info, rules_file_error)
        })
    }

    fn load_worktree_rules_file(
        worktree: Entity<Worktree>,
        project: Entity<Project>,
        cx: &mut App,
    ) -> Option<Task<Result<RulesFileContext>>> {
        let worktree_ref = worktree.read(cx);
        let worktree_id = worktree_ref.id();
        let selected_rules_file = RULES_FILE_NAMES
            .into_iter()
            .filter_map(|name| {
                worktree_ref
                    .entry_for_path(name)
                    .filter(|entry| entry.is_file())
                    .map(|entry| entry.path.clone())
            })
            .next();

        // Note that Cline supports `.clinerules` being a directory, but that is not currently
        // supported. This doesn't seem to occur often in GitHub repositories.
        selected_rules_file.map(|path_in_worktree| {
            let project_path = ProjectPath {
                worktree_id,
                path: path_in_worktree.clone(),
            };
            let buffer_task =
                project.update(cx, |project, cx| project.open_buffer(project_path, cx));
            let rope_task = cx.spawn(async move |cx| {
                buffer_task.await?.read_with(cx, |buffer, cx| {
                    let project_entry_id = buffer.entry_id(cx).context("buffer has no file")?;
                    anyhow::Ok((project_entry_id, buffer.as_rope().clone()))
                })?
            });
            // Build a string from the rope on a background thread.
            cx.background_spawn(async move {
                let (project_entry_id, rope) = rope_task.await?;
                anyhow::Ok(RulesFileContext {
                    path_in_worktree,
                    text: rope.to_string().trim().to_string(),
                    project_entry_id: project_entry_id.to_usize(),
                })
            })
        })
    }

    pub fn prompt_store(&self) -> &Option<Entity<PromptStore>> {
        &self.prompt_store
    }

    pub fn tools(&self) -> Entity<ToolWorkingSet> {
        self.tools.clone()
    }

    /// Returns the number of threads.
    pub fn thread_count(&self) -> usize {
        self.threads.len()
    }

    pub fn unordered_threads(&self) -> impl Iterator<Item = &SerialiCodeOrbitThreadMetadata> {
        self.threads.iter()
    }

    pub fn reverse_chronological_threads(&self) -> Vec<SerialiCodeOrbitThreadMetadata> {
        let mut threads = self.threads.iter().cloned().collect::<Vec<_>>();
        threads.sort_unstable_by_key(|thread| std::cmp::Reverse(thread.updated_at));
        threads
    }

    pub fn create_thread(&mut self, cx: &mut Context<Self>) -> Entity<Thread> {
        cx.new(|cx| {
            Thread::new(
                self.project.clone(),
                self.tools.clone(),
                self.prompt_builder.clone(),
                self.project_context.clone(),
                cx,
            )
        })
    }

    pub fn create_thread_from_serialiCodeOrbit(
        &mut self,
        serialiCodeOrbit: SerialiCodeOrbitThread,
        cx: &mut Context<Self>,
    ) -> Entity<Thread> {
        cx.new(|cx| {
            Thread::deserialize(
                ThreadId::new(),
                serialiCodeOrbit,
                self.project.clone(),
                self.tools.clone(),
                self.prompt_builder.clone(),
                self.project_context.clone(),
                None,
                cx,
            )
        })
    }

    pub fn open_thread(
        &self,
        id: &ThreadId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Result<Entity<Thread>>> {
        let id = id.clone();
        let database_future = ThreadsDatabase::global_future(cx);
        let this = cx.weak_entity();
        window.spawn(cx, async move |cx| {
            let database = database_future.await.map_err(|err| anyhow!(err))?;
            let thread = database
                .try_find_thread(id.clone())
                .await?
                .with_context(|| format!("no thread found with ID: {id:?}"))?;

            let thread = this.update_in(cx, |this, window, cx| {
                cx.new(|cx| {
                    Thread::deserialize(
                        id.clone(),
                        thread,
                        this.project.clone(),
                        this.tools.clone(),
                        this.prompt_builder.clone(),
                        this.project_context.clone(),
                        Some(window),
                        cx,
                    )
                })
            })?;

            Ok(thread)
        })
    }

    pub fn save_thread(&self, thread: &Entity<Thread>, cx: &mut Context<Self>) -> Task<Result<()>> {
        let (metadata, serialiCodeOrbit_thread) =
            thread.update(cx, |thread, cx| (thread.id().clone(), thread.serialize(cx)));

        let database_future = ThreadsDatabase::global_future(cx);
        cx.spawn(async move |this, cx| {
            let serialiCodeOrbit_thread = serialiCodeOrbit_thread.await?;
            let database = database_future.await.map_err(|err| anyhow!(err))?;
            database.save_thread(metadata, serialiCodeOrbit_thread).await?;

            this.update(cx, |this, cx| this.reload(cx))?.await
        })
    }

    pub fn delete_thread(&mut self, id: &ThreadId, cx: &mut Context<Self>) -> Task<Result<()>> {
        let id = id.clone();
        let database_future = ThreadsDatabase::global_future(cx);
        cx.spawn(async move |this, cx| {
            let database = database_future.await.map_err(|err| anyhow!(err))?;
            database.delete_thread(id.clone()).await?;

            this.update(cx, |this, cx| {
                this.threads.retain(|thread| thread.id != id);
                cx.notify();
            })
        })
    }

    pub fn reload(&self, cx: &mut Context<Self>) -> Task<Result<()>> {
        let database_future = ThreadsDatabase::global_future(cx);
        cx.spawn(async move |this, cx| {
            let threads = database_future
                .await
                .map_err(|err| anyhow!(err))?
                .list_threads()
                .await?;

            this.update(cx, |this, cx| {
                this.threads = threads;
                cx.notify();
            })
        })
    }

    fn load_default_profile(&self, cx: &mut Context<Self>) {
        let assistant_settings = AgentSettings::get_global(cx);

        self.load_profile_by_id(assistant_settings.default_profile.clone(), cx);
    }

    pub fn load_profile_by_id(&self, profile_id: AgentProfileId, cx: &mut Context<Self>) {
        let assistant_settings = AgentSettings::get_global(cx);

        if let Some(profile) = assistant_settings.profiles.get(&profile_id) {
            self.load_profile(profile.clone(), cx);
        }
    }

    pub fn load_profile(&self, profile: AgentProfile, cx: &mut Context<Self>) {
        self.tools.update(cx, |tools, cx| {
            tools.disable_all_tools(cx);
            tools.enable(
                ToolSource::Native,
                &profile
                    .tools
                    .into_iter()
                    .filter_map(|(tool, enabled)| enabled.then(|| tool))
                    .collect::<Vec<_>>(),
                cx,
            );
        });

        if profile.enable_all_context_servers {
            for context_server_id in self
                .project
                .read(cx)
                .context_server_store()
                .read(cx)
                .all_server_ids()
            {
                self.tools.update(cx, |tools, cx| {
                    tools.enable_source(
                        ToolSource::ContextServer {
                            id: context_server_id.0.into(),
                        },
                        cx,
                    );
                });
            }
            // Enable all the tools from all context servers, but disable the ones that are explicitly disabled
            for (context_server_id, preset) in profile.context_servers {
                self.tools.update(cx, |tools, cx| {
                    tools.disable(
                        ToolSource::ContextServer {
                            id: context_server_id.into(),
                        },
                        &preset
                            .tools
                            .into_iter()
                            .filter_map(|(tool, enabled)| (!enabled).then(|| tool))
                            .collect::<Vec<_>>(),
                        cx,
                    )
                })
            }
        } else {
            for (context_server_id, preset) in profile.context_servers {
                self.tools.update(cx, |tools, cx| {
                    tools.enable(
                        ToolSource::ContextServer {
                            id: context_server_id.into(),
                        },
                        &preset
                            .tools
                            .into_iter()
                            .filter_map(|(tool, enabled)| enabled.then(|| tool))
                            .collect::<Vec<_>>(),
                        cx,
                    )
                })
            }
        }
    }

    fn register_context_server_handlers(&self, cx: &mut Context<Self>) {
        cx.subscribe(
            &self.project.read(cx).context_server_store(),
            Self::handle_context_server_event,
        )
        .detach();
    }

    fn handle_context_server_event(
        &mut self,
        context_server_store: Entity<ContextServerStore>,
        event: &project::context_server_store::Event,
        cx: &mut Context<Self>,
    ) {
        let tool_working_set = self.tools.clone();
        match event {
            project::context_server_store::Event::ServerStatusChanged { server_id, status } => {
                match status {
                    ContextServerStatus::Running => {
                        if let Some(server) =
                            context_server_store.read(cx).get_running_server(server_id)
                        {
                            let context_server_manager = context_server_store.clone();
                            cx.spawn({
                                let server = server.clone();
                                let server_id = server_id.clone();
                                async move |this, cx| {
                                    let Some(protocol) = server.client() else {
                                        return;
                                    };

                                    if protocol.capable(context_server::protocol::ServerCapability::Tools) {
                                        if let Some(tools) = protocol.list_tools().await.log_err() {
                                            let tool_ids = tool_working_set
                                                .update(cx, |tool_working_set, _| {
                                                    tools
                                                        .tools
                                                        .into_iter()
                                                        .map(|tool| {
                                                            log::info!(
                                                                "registering context server tool: {:?}",
                                                                tool.name
                                                            );
                                                            tool_working_set.insert(Arc::new(
                                                                ContextServerTool::new(
                                                                    context_server_manager.clone(),
                                                                    server.id(),
                                                                    tool,
                                                                ),
                                                            ))
                                                        })
                                                        .collect::<Vec<_>>()
                                                })
                                                .log_err();

                                            if let Some(tool_ids) = tool_ids {
                                                this.update(cx, |this, cx| {
                                                    this.context_server_tool_ids
                                                        .insert(server_id, tool_ids);
                                                    this.load_default_profile(cx);
                                                })
                                                .log_err();
                                            }
                                        }
                                    }
                                }
                            })
                            .detach();
                        }
                    }
                    ContextServerStatus::Stopped | ContextServerStatus::Error(_) => {
                        if let Some(tool_ids) = self.context_server_tool_ids.remove(server_id) {
                            tool_working_set.update(cx, |tool_working_set, _| {
                                tool_working_set.remove(&tool_ids);
                            });
                            self.load_default_profile(cx);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialiCodeOrbitThreadMetadata {
    pub id: ThreadId,
    pub summary: SharedString,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerialiCodeOrbitThread {
    pub version: String,
    pub summary: SharedString,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<SerialiCodeOrbitMessage>,
    #[serde(default)]
    pub initial_project_snapshot: Option<Arc<ProjectSnapshot>>,
    #[serde(default)]
    pub cumulative_token_usage: TokenUsage,
    #[serde(default)]
    pub request_token_usage: Vec<TokenUsage>,
    #[serde(default)]
    pub detailed_summary_state: DetailedSummaryState,
    #[serde(default)]
    pub exceeded_window_error: Option<ExceededWindowError>,
    #[serde(default)]
    pub model: Option<SerialiCodeOrbitLanguageModel>,
    #[serde(default)]
    pub completion_mode: Option<CompletionMode>,
    #[serde(default)]
    pub tool_use_limit_reached: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerialiCodeOrbitLanguageModel {
    pub provider: String,
    pub model: String,
}

impl SerialiCodeOrbitThread {
    pub const VERSION: &'static str = "0.2.0";

    pub fn from_json(json: &[u8]) -> Result<Self> {
        let saved_thread_json = serde_json::from_slice::<serde_json::Value>(json)?;
        match saved_thread_json.get("version") {
            Some(serde_json::Value::String(version)) => match version.as_str() {
                SerialiCodeOrbitThreadV0_1_0::VERSION => {
                    let saved_thread =
                        serde_json::from_value::<SerialiCodeOrbitThreadV0_1_0>(saved_thread_json)?;
                    Ok(saved_thread.upgrade())
                }
                SerialiCodeOrbitThread::VERSION => Ok(serde_json::from_value::<SerialiCodeOrbitThread>(
                    saved_thread_json,
                )?),
                _ => anyhow::bail!("unrecogniCodeOrbit serialiCodeOrbit thread version: {version:?}"),
            },
            None => {
                let saved_thread =
                    serde_json::from_value::<LegacySerialiCodeOrbitThread>(saved_thread_json)?;
                Ok(saved_thread.upgrade())
            }
            version => anyhow::bail!("unrecogniCodeOrbit serialiCodeOrbit thread version: {version:?}"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerialiCodeOrbitThreadV0_1_0(
    // The structure did not change, so we are reusing the latest SerialiCodeOrbitThread.
    // When making the next version, make sure this points to SerialiCodeOrbitThreadV0_2_0
    SerialiCodeOrbitThread,
);

impl SerialiCodeOrbitThreadV0_1_0 {
    pub const VERSION: &'static str = "0.1.0";

    pub fn upgrade(self) -> SerialiCodeOrbitThread {
        debug_assert_eq!(SerialiCodeOrbitThread::VERSION, "0.2.0");

        let mut messages: Vec<SerialiCodeOrbitMessage> = Vec::with_capacity(self.0.messages.len());

        for message in self.0.messages {
            if message.role == Role::User && !message.tool_results.is_empty() {
                if let Some(last_message) = messages.last_mut() {
                    debug_assert!(last_message.role == Role::Assistant);

                    last_message.tool_results = message.tool_results;
                    continue;
                }
            }

            messages.push(message);
        }

        SerialiCodeOrbitThread { messages, ..self.0 }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerialiCodeOrbitMessage {
    pub id: MessageId,
    pub role: Role,
    #[serde(default)]
    pub segments: Vec<SerialiCodeOrbitMessageSegment>,
    #[serde(default)]
    pub tool_uses: Vec<SerialiCodeOrbitToolUse>,
    #[serde(default)]
    pub tool_results: Vec<SerialiCodeOrbitToolResult>,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub creases: Vec<SerialiCodeOrbitCrease>,
    #[serde(default)]
    pub is_hidden: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SerialiCodeOrbitMessageSegment {
    #[serde(rename = "text")]
    Text {
        text: String,
    },
    #[serde(rename = "thinking")]
    Thinking {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    RedactedThinking {
        data: Vec<u8>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerialiCodeOrbitToolUse {
    pub id: LanguageModelToolUseId,
    pub name: SharedString,
    pub input: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerialiCodeOrbitToolResult {
    pub tool_use_id: LanguageModelToolUseId,
    pub is_error: bool,
    pub content: LanguageModelToolResultContent,
    pub output: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct LegacySerialiCodeOrbitThread {
    pub summary: SharedString,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<LegacySerialiCodeOrbitMessage>,
    #[serde(default)]
    pub initial_project_snapshot: Option<Arc<ProjectSnapshot>>,
}

impl LegacySerialiCodeOrbitThread {
    pub fn upgrade(self) -> SerialiCodeOrbitThread {
        SerialiCodeOrbitThread {
            version: SerialiCodeOrbitThread::VERSION.to_string(),
            summary: self.summary,
            updated_at: self.updated_at,
            messages: self.messages.into_iter().map(|msg| msg.upgrade()).collect(),
            initial_project_snapshot: self.initial_project_snapshot,
            cumulative_token_usage: TokenUsage::default(),
            request_token_usage: Vec::new(),
            detailed_summary_state: DetailedSummaryState::default(),
            exceeded_window_error: None,
            model: None,
            completion_mode: None,
            tool_use_limit_reached: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LegacySerialiCodeOrbitMessage {
    pub id: MessageId,
    pub role: Role,
    pub text: String,
    #[serde(default)]
    pub tool_uses: Vec<SerialiCodeOrbitToolUse>,
    #[serde(default)]
    pub tool_results: Vec<SerialiCodeOrbitToolResult>,
}

impl LegacySerialiCodeOrbitMessage {
    fn upgrade(self) -> SerialiCodeOrbitMessage {
        SerialiCodeOrbitMessage {
            id: self.id,
            role: self.role,
            segments: vec![SerialiCodeOrbitMessageSegment::Text { text: self.text }],
            tool_uses: self.tool_uses,
            tool_results: self.tool_results,
            context: String::new(),
            creases: Vec::new(),
            is_hidden: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerialiCodeOrbitCrease {
    pub start: usize,
    pub end: usize,
    pub icon_path: SharedString,
    pub label: SharedString,
}

struct GlobalThreadsDatabase(
    Shared<BoxFuture<'static, Result<Arc<ThreadsDatabase>, Arc<anyhow::Error>>>>,
);

impl Global for GlobalThreadsDatabase {}

pub(crate) struct ThreadsDatabase {
    executor: BackgroundExecutor,
    connection: Arc<Mutex<Connection>>,
}

impl ThreadsDatabase {
    fn connection(&self) -> Arc<Mutex<Connection>> {
        self.connection.clone()
    }

    const COMPRESSION_LEVEL: i32 = 3;
}

impl Bind for ThreadId {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        self.to_string().bind(statement, start_index)
    }
}

impl Column for ThreadId {
    fn column(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)> {
        let (id_str, next_index) = String::column(statement, start_index)?;
        Ok((ThreadId::from(id_str.as_str()), next_index))
    }
}

impl ThreadsDatabase {
    fn global_future(
        cx: &mut App,
    ) -> Shared<BoxFuture<'static, Result<Arc<ThreadsDatabase>, Arc<anyhow::Error>>>> {
        GlobalThreadsDatabase::global(cx).0.clone()
    }

    fn init(cx: &mut App) {
        let executor = cx.background_executor().clone();
        let database_future = executor
            .spawn({
                let executor = executor.clone();
                let threads_dir = paths::data_dir().join("threads");
                async move { ThreadsDatabase::new(threads_dir, executor) }
            })
            .then(|result| future::ready(result.map(Arc::new).map_err(Arc::new)))
            .boxed()
            .shared();

        cx.set_global(GlobalThreadsDatabase(database_future));
    }

    pub fn new(threads_dir: PathBuf, executor: BackgroundExecutor) -> Result<Self> {
        std::fs::create_dir_all(&threads_dir)?;

        let sqlite_path = threads_dir.join("threads.db");
        let mdb_path = threads_dir.join("threads-db.1.mdb");

        let needs_migration_from_heed = mdb_path.exists();

        let connection = Connection::open_file(&sqlite_path.to_string_lossy());

        connection.exec(indoc! {"
                CREATE TABLE IF NOT EXISTS threads (
                    id TEXT PRIMARY KEY,
                    summary TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    data_type TEXT NOT NULL,
                    data BLOB NOT NULL
                )
            "})?()
        .map_err(|e| anyhow!("Failed to create threads table: {}", e))?;

        let db = Self {
            executor: executor.clone(),
            connection: Arc::new(Mutex::new(connection)),
        };

        if needs_migration_from_heed {
            let db_connection = db.connection();
            let executor_clone = executor.clone();
            executor
                .spawn(async move {
                    log::info!("Starting threads.db migration");
                    Self::migrate_from_heed(&mdb_path, db_connection, executor_clone)?;
                    std::fs::remove_dir_all(mdb_path)?;
                    log::info!("threads.db migrated to sqlite");
                    Ok::<(), anyhow::Error>(())
                })
                .detach();
        }

        Ok(db)
    }

    // Remove this migration after 2025-09-01
    fn migrate_from_heed(
        mdb_path: &Path,
        connection: Arc<Mutex<Connection>>,
        _executor: BackgroundExecutor,
    ) -> Result<()> {
        use heed::types::SerdeBincode;
        struct SerialiCodeOrbitThreadHeed(SerialiCodeOrbitThread);

        impl heed::BytesEncode<'_> for SerialiCodeOrbitThreadHeed {
            type EItem = SerialiCodeOrbitThreadHeed;

            fn bytes_encode(
                item: &Self::EItem,
            ) -> Result<std::borrow::Cow<[u8]>, heed::BoxedError> {
                serde_json::to_vec(&item.0)
                    .map(std::borrow::Cow::Owned)
                    .map_err(Into::into)
            }
        }

        impl<'a> heed::BytesDecode<'a> for SerialiCodeOrbitThreadHeed {
            type DItem = SerialiCodeOrbitThreadHeed;

            fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, heed::BoxedError> {
                SerialiCodeOrbitThread::from_json(bytes)
                    .map(SerialiCodeOrbitThreadHeed)
                    .map_err(Into::into)
            }
        }

        const ONE_GB_IN_BYTES: usize = 1024 * 1024 * 1024;

        let env = unsafe {
            heed::EnvOpenOptions::new()
                .map_size(ONE_GB_IN_BYTES)
                .max_dbs(1)
                .open(mdb_path)?
        };

        let txn = env.write_txn()?;
        let threads: heed::Database<SerdeBincode<ThreadId>, SerialiCodeOrbitThreadHeed> = env
            .open_database(&txn, Some("threads"))?
            .ok_or_else(|| anyhow!("threads database not found"))?;

        for result in threads.iter(&txn)? {
            let (thread_id, thread_heed) = result?;
            Self::save_thread_sync(&connection, thread_id, thread_heed.0)?;
        }

        Ok(())
    }

    fn save_thread_sync(
        connection: &Arc<Mutex<Connection>>,
        id: ThreadId,
        thread: SerialiCodeOrbitThread,
    ) -> Result<()> {
        let json_data = serde_json::to_string(&thread)?;
        let summary = thread.summary.to_string();
        let updated_at = thread.updated_at.to_rfc3339();

        let connection = connection.lock().unwrap();

        let compressed = zstd::encode_all(json_data.as_bytes(), Self::COMPRESSION_LEVEL)?;
        let data_type = DataType::Zstd;
        let data = compressed;

        let mut insert = connection.exec_bound::<(ThreadId, String, String, DataType, Vec<u8>)>(indoc! {"
            INSERT OR REPLACE INTO threads (id, summary, updated_at, data_type, data) VALUES (?, ?, ?, ?, ?)
        "})?;

        insert((id, summary, updated_at, data_type, data))?;

        Ok(())
    }

    pub fn list_threads(&self) -> Task<Result<Vec<SerialiCodeOrbitThreadMetadata>>> {
        let connection = self.connection.clone();

        self.executor.spawn(async move {
            let connection = connection.lock().unwrap();
            let mut select =
                connection.select_bound::<(), (ThreadId, String, String)>(indoc! {"
                SELECT id, summary, updated_at FROM threads ORDER BY updated_at DESC
            "})?;

            let rows = select(())?;
            let mut threads = Vec::new();

            for (id, summary, updated_at) in rows {
                threads.push(SerialiCodeOrbitThreadMetadata {
                    id,
                    summary: summary.into(),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
                });
            }

            Ok(threads)
        })
    }

    pub fn try_find_thread(&self, id: ThreadId) -> Task<Result<Option<SerialiCodeOrbitThread>>> {
        let connection = self.connection.clone();

        self.executor.spawn(async move {
            let connection = connection.lock().unwrap();
            let mut select = connection.select_bound::<ThreadId, (DataType, Vec<u8>)>(indoc! {"
                SELECT data_type, data FROM threads WHERE id = ? LIMIT 1
            "})?;

            let rows = select(id)?;
            if let Some((data_type, data)) = rows.into_iter().next() {
                let json_data = match data_type {
                    DataType::Zstd => {
                        let decompressed = zstd::decode_all(&data[..])?;
                        String::from_utf8(decompressed)?
                    }
                    DataType::Json => String::from_utf8(data)?,
                };

                let thread = SerialiCodeOrbitThread::from_json(json_data.as_bytes())?;
                Ok(Some(thread))
            } else {
                Ok(None)
            }
        })
    }

    pub fn save_thread(&self, id: ThreadId, thread: SerialiCodeOrbitThread) -> Task<Result<()>> {
        let connection = self.connection.clone();

        self.executor
            .spawn(async move { Self::save_thread_sync(&connection, id, thread) })
    }

    pub fn delete_thread(&self, id: ThreadId) -> Task<Result<()>> {
        let connection = self.connection.clone();

        self.executor.spawn(async move {
            let connection = connection.lock().unwrap();

            let mut delete = connection.exec_bound::<ThreadId>(indoc! {"
                DELETE FROM threads WHERE id = ?
            "})?;

            delete(id)?;

            Ok(())
        })
    }
}
