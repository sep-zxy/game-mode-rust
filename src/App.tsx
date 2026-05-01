import { useEffect, useMemo, useState } from "react";
import {
  disableMode,
  enableMode,
  getClashStatus,
  getConfig,
  listRunningProcesses,
  refreshTray,
  saveConfig,
  selectExecutablePath,
  switchActivePreset,
  testCloseApps,
  testStartApps,
} from "./api/tauri";
import {
  ActionListKey,
  AppEntry,
  ClashOptions,
  ConfigV2,
  Preset,
  ProcessInfo,
} from "./types";

type EditorTab = "rules" | "apps";
type ActionGroup = "enable" | "disable";

const ACTION_GROUPS: Record<ActionGroup, ActionListKey[]> = {
  enable: ["enable_close", "enable_start"],
  disable: ["disable_start", "disable_close"],
};

const ACTION_GROUP_LABELS: Record<ActionGroup, string> = {
  enable: "开启模式",
  disable: "关闭模式",
};

const ACTION_LABEL_TEMPLATE: Record<ActionListKey, string> = {
  enable_close: "开启模式关闭",
  enable_start: "开启模式开启",
  disable_start: "关闭模式开启",
  disable_close: "关闭模式关闭",
};

const ACTION_HINT_TEMPLATE: Record<ActionListKey, string> = {
  enable_close: "进入游戏前需要先关掉的程序",
  enable_start: "进入游戏前需要一起拉起的程序",
  disable_start: "退出游戏模式后需要恢复启动的程序",
  disable_close: "退出游戏模式后顺手关掉的程序",
};

const AUTO_COPY_MAP: Partial<Record<ActionListKey, ActionListKey>> = {
  enable_close: "disable_start",
  enable_start: "disable_close",
};

const RULE_ITEMS: Array<{ key: keyof ClashOptions; label: string }> = [
  { key: "enable_manage_clash", label: "开启预设时执行 Clash 控制" },
  { key: "enable_disable_tun", label: "开启预设时关闭 Tun" },
  { key: "enable_disable_system_proxy", label: "开启预设时关闭系统代理" },
  { key: "disable_manage_clash", label: "关闭预设时执行 Clash 恢复" },
  { key: "disable_restore_tun", label: "关闭预设时恢复 Tun" },
  { key: "disable_restore_system_proxy", label: "关闭预设时恢复系统代理" },
  { key: "disable_start_clash_if_needed", label: "关闭预设时 Clash 未运行则自动启动" },
];

const DEFAULT_CLASH_OPTIONS: ClashOptions = {
  enable_manage_clash: true,
  enable_disable_tun: true,
  enable_disable_system_proxy: true,
  disable_manage_clash: true,
  disable_restore_tun: true,
  disable_restore_system_proxy: true,
  disable_start_clash_if_needed: true,
};

const EMPTY_EDITOR = {
  listKey: null as ActionListKey | null,
  index: -1,
};

function parseArgs(text: string): string[] {
  return text
    .trim()
    .split(/\s+/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function stringifyArgs(args: string[]): string {
  return args.join(" ");
}

function createPreset(name: string): Preset {
  const id =
    typeof crypto !== "undefined" && "randomUUID" in crypto
      ? `preset-${crypto.randomUUID().slice(0, 8)}`
      : `preset-${Date.now()}`;

  return {
    id,
    name,
    enable_close: [],
    enable_start: [],
    disable_start: [],
    disable_close: [],
    clash_options: { ...DEFAULT_CLASH_OPTIONS },
  };
}

function cloneAppEntry(item: AppEntry): AppEntry {
  return {
    alias: item.alias,
    name: item.name,
    path: item.path,
    start_args: [...item.start_args],
  };
}

function clonePreset(preset: Preset): Preset {
  return {
    ...preset,
    enable_close: preset.enable_close.map(cloneAppEntry),
    enable_start: preset.enable_start.map(cloneAppEntry),
    disable_start: preset.disable_start.map(cloneAppEntry),
    disable_close: preset.disable_close.map(cloneAppEntry),
    clash_options: { ...preset.clash_options },
  };
}

function dedupeAppsByPath(items: AppEntry[]): AppEntry[] {
  const seen = new Set<string>();
  return items.filter((item) => {
    const key = item.path.toLowerCase();
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
}

function toTunText(value: unknown | null): string {
  if (value == null) {
    return "未知";
  }
  if (typeof value === "boolean") {
    return value ? "已开启" : "已关闭";
  }
  if (typeof value === "object" && "enable" in value) {
    return Boolean((value as { enable?: unknown }).enable) ? "已开启" : "已关闭";
  }
  return String(value);
}

function createAppEntry(alias: string, name: string, path: string, argsText: string): AppEntry {
  const nextName = name.trim();
  return {
    alias: alias.trim() || nextName,
    name: nextName,
    path: path.trim(),
    start_args: parseArgs(argsText),
  };
}

function getDisplayName(item: AppEntry): string {
  return item.alias.trim() || item.name;
}

function App() {
  const [config, setConfig] = useState<ConfigV2 | null>(null);
  const [selectedPresetId, setSelectedPresetId] = useState("");
  const [tab, setTab] = useState<EditorTab>("apps");
  const [actionGroup, setActionGroup] = useState<ActionGroup>("enable");
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("正在加载配置...");
  const [clashRuntimeStatus, setClashRuntimeStatus] = useState("Tun: 未查询 · System Proxy: 未查询");
  const [showAddPanel, setShowAddPanel] = useState(false);
  const [addTargetKey, setAddTargetKey] = useState<ActionListKey>("enable_close");
  const [editingTarget, setEditingTarget] = useState(EMPTY_EDITOR);

  const [newAppAlias, setNewAppAlias] = useState("");
  const [newAppName, setNewAppName] = useState("");
  const [newAppPath, setNewAppPath] = useState("");
  const [newAppArgs, setNewAppArgs] = useState("");
  const [runningProcesses, setRunningProcesses] = useState<ProcessInfo[]>([]);
  const [selectedProcessPath, setSelectedProcessPath] = useState("");

  const selectedPreset = useMemo(() => {
    if (!config) {
      return null;
    }
    return config.presets.find((item) => item.id === selectedPresetId) ?? null;
  }, [config, selectedPresetId]);

  const activePreset = useMemo(() => {
    if (!config) {
      return null;
    }
    return config.presets.find((item) => item.id === config.active_preset_id) ?? null;
  }, [config]);

  const visibleListKeys = ACTION_GROUPS[actionGroup];
  const editingItem = useMemo(() => {
    if (!selectedPreset || !editingTarget.listKey || editingTarget.index < 0) {
      return null;
    }
    return selectedPreset[editingTarget.listKey][editingTarget.index] ?? null;
  }, [selectedPreset, editingTarget]);

  async function refreshConfig(): Promise<ConfigV2> {
    const next = await getConfig();
    setConfig(next);
    setSelectedPresetId((prev) => {
      if (prev && next.presets.some((item) => item.id === prev)) {
        return prev;
      }
      return next.active_preset_id;
    });
    return next;
  }

  useEffect(() => {
    refreshConfig()
      .then((next) => setStatus(next.runtime.last_error || "配置已加载"))
      .catch((error) => setStatus(`读取配置失败: ${String(error)}`));
  }, []);

  useEffect(() => {
    const firstKey = ACTION_GROUPS[actionGroup][0];
    setAddTargetKey((prev) => (ACTION_GROUPS[actionGroup].includes(prev) ? prev : firstKey));
    setEditingTarget((prev) =>
      prev.listKey && ACTION_GROUPS[actionGroup].includes(prev.listKey) ? prev : EMPTY_EDITOR
    );
  }, [actionGroup]);

  function updateSelectedPreset(mutator: (preset: Preset) => void) {
    setConfig((prev) => {
      if (!prev) {
        return prev;
      }

      const presets = prev.presets.map((item) => {
        if (item.id !== selectedPresetId) {
          return item;
        }
        const clone = clonePreset(item);
        mutator(clone);
        return clone;
      });

      return { ...prev, presets };
    });
  }

  async function persistConfig(successMessage: string) {
    if (!config) {
      return;
    }

    setBusy(true);
    try {
      await saveConfig(config);
      await refreshTray();
      await refreshConfig();
      setStatus(successMessage);
    } catch (error) {
      setStatus(`保存失败: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function handleSetActivePreset() {
    if (!selectedPreset) {
      return;
    }
    setBusy(true);
    try {
      await switchActivePreset(selectedPreset.id);
      await refreshTray();
      await refreshConfig();
      setStatus(`已设为激活预设：${selectedPreset.name}`);
    } catch (error) {
      setStatus(`切换激活预设失败: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function handleEnableMode() {
    setBusy(true);
    try {
      const report = await enableMode();
      await refreshTray();
      await refreshConfig();
      if (report.clash_status) {
        setClashRuntimeStatus(
          `Tun: ${toTunText(report.clash_status.tun)} · System Proxy: ${
            report.clash_status.system_proxy == null
              ? "未知"
              : report.clash_status.system_proxy
                ? "开启"
                : "关闭"
          }`
        );
      }
      setStatus(`已开启模式：${report.preset_name}`);
    } catch (error) {
      setStatus(`开启模式失败: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function handleDisableMode() {
    setBusy(true);
    try {
      const report = await disableMode();
      await refreshTray();
      await refreshConfig();
      if (report.clash_status) {
        setClashRuntimeStatus(
          `Tun: ${toTunText(report.clash_status.tun)} · System Proxy: ${
            report.clash_status.system_proxy == null
              ? "未知"
              : report.clash_status.system_proxy
                ? "开启"
                : "关闭"
          }`
        );
      }
      setStatus(`已关闭模式：${report.preset_name}`);
    } catch (error) {
      setStatus(`关闭模式失败: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function handleFetchClashStatus() {
    setBusy(true);
    try {
      const clashStatus = await getClashStatus();
      setClashRuntimeStatus(
        `Tun: ${toTunText(clashStatus.tun)} · System Proxy: ${
          clashStatus.system_proxy == null ? "未知" : clashStatus.system_proxy ? "开启" : "关闭"
        }`
      );
      setStatus("已获取 Clash 运行状态");
    } catch (error) {
      setStatus(`读取 Clash 状态失败: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  function handleAddPreset() {
    setConfig((prev) => {
      if (!prev) {
        return prev;
      }
      const item = createPreset(`预设 ${prev.presets.length + 1}`);
      setSelectedPresetId(item.id);
      return { ...prev, presets: [...prev.presets, item] };
    });
    setStatus("已新增空预设，请保存");
  }

  function handleClonePreset() {
    if (!selectedPreset) {
      return;
    }
    setConfig((prev) => {
      if (!prev) {
        return prev;
      }
      const cloned = clonePreset(selectedPreset);
      cloned.id = createPreset("tmp").id;
      cloned.name = `${selectedPreset.name} - 副本`;
      setSelectedPresetId(cloned.id);
      return { ...prev, presets: [...prev.presets, cloned] };
    });
    setStatus("已基于当前预设创建副本，请保存");
  }

  function handleDeletePreset() {
    if (!config || config.presets.length <= 1) {
      setStatus("至少保留一个预设，无法删除");
      return;
    }

    setConfig((prev) => {
      if (!prev) {
        return prev;
      }
      const remain = prev.presets.filter((item) => item.id !== selectedPresetId);
      const nextSelected = remain[0]?.id ?? "";
      setSelectedPresetId(nextSelected);

      return {
        ...prev,
        presets: remain,
        active_preset_id:
          prev.active_preset_id === selectedPresetId ? nextSelected : prev.active_preset_id,
      };
    });
    setEditingTarget(EMPTY_EDITOR);
    setStatus("已删除预设，请保存");
  }

  function handleRuleToggle(key: keyof ClashOptions, checked: boolean) {
    updateSelectedPreset((target) => {
      target.clash_options[key] = checked;
    });
  }

  function handlePresetNameChange(value: string) {
    updateSelectedPreset((target) => {
      target.name = value;
    });
  }

  async function handleBrowseClashPath() {
    setBusy(true);
    try {
      const filePath = await selectExecutablePath();
      if (!filePath) {
        return;
      }
      setConfig((prev) =>
        prev ? { ...prev, global: { ...prev.global, clash_path: filePath } } : prev
      );
      setStatus("已选择 Clash 可执行路径，请点击保存 Clash 设置");
    } catch (error) {
      setStatus(`选择可执行路径失败: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function handleSaveClashSettings() {
    if (!config) {
      return;
    }
    const port = config.global.clash_port;
    if (!Number.isInteger(port) || port < 1 || port > 65535) {
      setStatus("控制端口必须是 1-65535 的整数");
      return;
    }
    await persistConfig("Clash 全局设置已保存");
  }

  function resetAddForm() {
    setNewAppAlias("");
    setNewAppName("");
    setNewAppPath("");
    setNewAppArgs("");
    setSelectedProcessPath("");
    setRunningProcesses([]);
  }

  function handleOpenAddPanel() {
    setShowAddPanel(true);
    setAddTargetKey(visibleListKeys[0]);
  }

  function handleCloseAddPanel() {
    setShowAddPanel(false);
    resetAddForm();
  }

  function addAppToList(listKey: ActionListKey, app: AppEntry) {
    updateSelectedPreset((target) => {
      target[listKey] = dedupeAppsByPath([...target[listKey], app]);
      const pair = AUTO_COPY_MAP[listKey];
      if (pair) {
        target[pair] = dedupeAppsByPath([...target[pair], cloneAppEntry(app)]);
      }
    });
  }

  function handleAddManualApp() {
    if (!newAppName.trim() || !newAppPath.trim()) {
      setStatus("进程名和可执行路径不能为空");
      return;
    }

    addAppToList(addTargetKey, createAppEntry(newAppAlias, newAppName, newAppPath, newAppArgs));
    handleCloseAddPanel();
    setStatus(`已添加到 ${ACTION_LABEL_TEMPLATE[addTargetKey]} 列表`);
  }

  async function handleLoadRunningProcesses() {
    setBusy(true);
    try {
      const list = await listRunningProcesses();
      setRunningProcesses(list);
      setSelectedProcessPath(list[0]?.path ?? "");
      setStatus(`已读取运行中进程：${list.length} 项`);
    } catch (error) {
      setStatus(`读取运行进程失败: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  function handleAddProcessApp() {
    const proc = runningProcesses.find((item) => item.path === selectedProcessPath);
    if (!proc) {
      setStatus("未选择可添加的进程");
      return;
    }

    addAppToList(addTargetKey, {
      alias: proc.name,
      name: proc.name,
      path: proc.path,
      start_args: [],
    });
    handleCloseAddPanel();
    setStatus(`已将 ${proc.name} 添加到 ${ACTION_LABEL_TEMPLATE[addTargetKey]} 列表`);
  }

  function handleRemoveApp(listKey: ActionListKey, index: number) {
    updateSelectedPreset((target) => {
      target[listKey] = target[listKey].filter((_, itemIndex) => itemIndex !== index);
    });
    setEditingTarget((prev) =>
      prev.listKey === listKey && prev.index === index ? EMPTY_EDITOR : prev
    );
    setStatus(`已从 ${ACTION_LABEL_TEMPLATE[listKey]} 列表移除程序`);
  }

  function openEditor(listKey: ActionListKey, index: number) {
    setEditingTarget({ listKey, index });
  }

  function updateEditingItem(mutator: (item: AppEntry) => AppEntry) {
    if (!editingTarget.listKey || editingTarget.index < 0) {
      return;
    }

    const listKey = editingTarget.listKey;
    const targetIndex = editingTarget.index;

    updateSelectedPreset((target) => {
      target[listKey] = target[listKey].map((item, index) => (index === targetIndex ? mutator(item) : item));
    });
  }

  async function handleTestStartList(listKey: ActionListKey) {
    setBusy(true);
    try {
      await testStartApps(listKey);
      setStatus(`${ACTION_LABEL_TEMPLATE[listKey]} 列表启动测试已执行`);
    } catch (error) {
      setStatus(`启动测试失败: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function handleTestCloseList(listKey: ActionListKey) {
    setBusy(true);
    try {
      await testCloseApps(listKey);
      setStatus(`${ACTION_LABEL_TEMPLATE[listKey]} 列表关闭测试已执行`);
    } catch (error) {
      setStatus(`关闭测试失败: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  function renderListCard(listKey: ActionListKey) {
    if (!selectedPreset) {
      return null;
    }

    const items = selectedPreset[listKey];
    const allowStartTest = listKey === "enable_start" || listKey === "disable_start";
    const allowCloseTest = listKey === "enable_close" || listKey === "disable_close";

    return (
      <section className="mode-list-card" key={listKey}>
        <div className="list-card-head">
          <div>
            <h3>{ACTION_LABEL_TEMPLATE[listKey]}</h3>
            <p>{ACTION_HINT_TEMPLATE[listKey]}</p>
          </div>
          <div className="list-card-actions">
            {allowCloseTest ? (
              <button className="btn ghost" onClick={() => handleTestCloseList(listKey)} disabled={busy}>
                测试关闭
              </button>
            ) : null}
            {allowStartTest ? (
              <button className="btn ghost" onClick={() => handleTestStartList(listKey)} disabled={busy}>
                测试启动
              </button>
            ) : null}
          </div>
        </div>

        <div className="simple-list">
          {items.length === 0 ? (
            <div className="empty-list-card">当前列表为空</div>
          ) : (
            items.map((item, index) => (
              <article className="simple-list-item" key={`${listKey}-${item.path}-${index}`}>
                <div className="simple-list-text">
                  <strong>{getDisplayName(item)}</strong>
                  <span>参数：{stringifyArgs(item.start_args) || "无"}</span>
                </div>
                <div className="simple-list-actions">
                  <button className="btn ghost small" onClick={() => openEditor(listKey, index)}>
                    修改
                  </button>
                  <button className="btn danger small" onClick={() => handleRemoveApp(listKey, index)}>
                    删除
                  </button>
                </div>
              </article>
            ))
          )}
        </div>
      </section>
    );
  }

  if (!config || !selectedPreset) {
    return <div className="loading">正在加载配置...</div>;
  }

  return (
    <div className="app-shell">
      <section className="card hero-card">
        <div>
          <h1>GameMode 预设工作台</h1>
          <p>应用编排先按开启/关闭模式分组，列表默认简化展示，真要细改再展开，别整得满屏都是输入框。</p>
        </div>
        <div className="hero-meta">
          <span className="badge">
            当前激活预设：<strong>{activePreset?.name ?? "-"}</strong>
          </span>
          <span className={config.runtime.mode_active ? "badge danger" : "badge success"}>
            模式状态：{config.runtime.mode_active ? "已开启" : "已关闭"}
          </span>
        </div>
        <div className="hero-actions">
          <button className="btn primary" onClick={handleSetActivePreset} disabled={busy}>
            设为激活
          </button>
          <button className="btn secondary" onClick={handleEnableMode} disabled={busy || config.runtime.mode_active}>
            开启模式
          </button>
          <button className="btn secondary" onClick={handleDisableMode} disabled={busy || !config.runtime.mode_active}>
            关闭模式
          </button>
          <button className="btn ghost" onClick={handleAddPreset} disabled={busy}>
            新增空白预设
          </button>
          <button className="btn ghost" onClick={handleClonePreset} disabled={busy}>
            基于当前预设新建
          </button>
        </div>
      </section>

      <div className="workspace-grid">
        <aside className="card preset-panel">
          <div className="panel-head">
            <h2>预设库</h2>
            <p>左侧挑预设，右侧改规则和编排。</p>
          </div>

          <div className="preset-list">
            {config.presets.map((item) => {
              const selected = item.id === selectedPresetId;
              const active = item.id === config.active_preset_id;
              return (
                <button
                  key={item.id}
                  className={selected ? "preset-item selected" : "preset-item"}
                  onClick={() => setSelectedPresetId(item.id)}
                >
                  <span className={active ? "dot active" : "dot"} />
                  <span>{item.name}</span>
                </button>
              );
            })}
          </div>

          <div className="preset-actions">
            <button className="btn primary" onClick={handleAddPreset} disabled={busy}>
              新增
            </button>
            <button className="btn ghost" onClick={handleClonePreset} disabled={busy}>
              基于当前新建
            </button>
            <button className="btn ghost" onClick={handleDeletePreset} disabled={busy}>
              删除
            </button>
          </div>
        </aside>

        <div className="right-column">
          <section className="card editor-panel">
            <div className="preset-name-row">
              <label htmlFor="presetName">预设名称</label>
              <input
                id="presetName"
                value={selectedPreset.name}
                onChange={(event) => handlePresetNameChange(event.target.value)}
              />
              <button className="btn primary" onClick={() => persistConfig("预设名称已保存")} disabled={busy}>
                保存预设
              </button>
            </div>

            <p className="editing-tip">
              正在编辑：{selectedPreset.name}（当前激活：{activePreset?.name ?? "-"}）
            </p>

            <div className="editor-tabs">
              <button className={tab === "rules" ? "tab-btn active" : "tab-btn"} onClick={() => setTab("rules")}>
                模式规则
              </button>
              <button className={tab === "apps" ? "tab-btn active" : "tab-btn"} onClick={() => setTab("apps")}>
                应用编排
              </button>
            </div>

            {tab === "rules" ? (
              <div className="rules-panel">
                <div className="rule-list">
                  {RULE_ITEMS.map((item) => (
                    <label className="rule-item" key={item.key}>
                      <input
                        type="checkbox"
                        checked={selectedPreset.clash_options[item.key]}
                        onChange={(event) => handleRuleToggle(item.key, event.target.checked)}
                      />
                      <span>{item.label}</span>
                    </label>
                  ))}
                </div>
                <div className="align-right">
                  <button className="btn primary" onClick={() => persistConfig("模式规则已保存")} disabled={busy}>
                    保存规则
                  </button>
                </div>
              </div>
            ) : (
              <div className="apps-panel">
                <div className="apps-toolbar">
                  <div className="apps-row list-row">
                    <label>动作列表</label>
                    <select value={actionGroup} onChange={(event) => setActionGroup(event.target.value as ActionGroup)}>
                      <option value="enable">{ACTION_GROUP_LABELS.enable}</option>
                      <option value="disable">{ACTION_GROUP_LABELS.disable}</option>
                    </select>
                  </div>

                  <div className="apps-toolbar-actions">
                    <button className="btn primary" onClick={handleOpenAddPanel} disabled={busy}>
                      新增
                    </button>
                    <button className="btn ghost" onClick={() => persistConfig("应用编排已保存")} disabled={busy}>
                      保存应用编排
                    </button>
                  </div>
                </div>

                {showAddPanel ? (
                  <section className="inline-editor">
                    <div className="inline-editor-head">
                      <h3>新增程序</h3>
                      <button className="btn ghost small" onClick={handleCloseAddPanel}>
                        收起
                      </button>
                    </div>

                    <div className="apps-row composer-target-row">
                      <label>添加到</label>
                      <select value={addTargetKey} onChange={(event) => setAddTargetKey(event.target.value as ActionListKey)}>
                        {visibleListKeys.map((key) => (
                          <option key={key} value={key}>
                            {ACTION_LABEL_TEMPLATE[key]}
                          </option>
                        ))}
                      </select>
                    </div>

                    <div className="apps-row app-form">
                      <input
                        placeholder="名称，默认显示别名"
                        value={newAppAlias}
                        onChange={(event) => setNewAppAlias(event.target.value)}
                      />
                      <input
                        placeholder="进程名，例如 Discord.exe"
                        value={newAppName}
                        onChange={(event) => setNewAppName(event.target.value)}
                      />
                      <input
                        placeholder="可执行路径，例如 C:/Apps/Discord/Discord.exe"
                        value={newAppPath}
                        onChange={(event) => setNewAppPath(event.target.value)}
                      />
                      <input
                        placeholder="启动参数（可选）"
                        value={newAppArgs}
                        onChange={(event) => setNewAppArgs(event.target.value)}
                      />
                      <button className="btn ghost" onClick={handleAddManualApp} disabled={busy}>
                        添加
                      </button>
                    </div>

                    <div className="apps-row process-row">
                      <button className="btn ghost" onClick={handleLoadRunningProcesses} disabled={busy}>
                        读取运行进程
                      </button>
                      <select value={selectedProcessPath} onChange={(event) => setSelectedProcessPath(event.target.value)}>
                        <option value="">请选择进程</option>
                        {runningProcesses.map((item) => (
                          <option key={item.path} value={item.path}>
                            {item.name} · {item.path}
                          </option>
                        ))}
                      </select>
                      <button className="btn ghost" onClick={handleAddProcessApp} disabled={busy}>
                        添加当前进程
                      </button>
                    </div>
                  </section>
                ) : null}

                <div className="mode-grid">
                  {visibleListKeys.map((key) => renderListCard(key))}
                </div>

                {editingItem && editingTarget.listKey ? (
                  <section className="inline-editor detail-editor">
                    <div className="inline-editor-head">
                      <div>
                        <h3>详细修改</h3>
                        <p>{ACTION_LABEL_TEMPLATE[editingTarget.listKey]}</p>
                      </div>
                      <button className="btn ghost small" onClick={() => setEditingTarget(EMPTY_EDITOR)}>
                        收起
                      </button>
                    </div>

                    <div className="detail-grid">
                      <label>
                        名称
                        <input
                          value={editingItem.alias}
                          placeholder={editingItem.name}
                          onChange={(event) =>
                            updateEditingItem((item) => ({ ...item, alias: event.target.value }))
                          }
                        />
                      </label>
                      <label>
                        进程名
                        <input
                          value={editingItem.name}
                          onChange={(event) =>
                            updateEditingItem((item) => ({ ...item, name: event.target.value }))
                          }
                        />
                      </label>
                      <label className="span-2">
                        可执行路径
                        <input
                          value={editingItem.path}
                          onChange={(event) =>
                            updateEditingItem((item) => ({ ...item, path: event.target.value }))
                          }
                        />
                      </label>
                      <label className="span-2">
                        启动参数
                        <input
                          value={stringifyArgs(editingItem.start_args)}
                          onChange={(event) =>
                            updateEditingItem((item) => ({
                              ...item,
                              start_args: parseArgs(event.target.value),
                            }))
                          }
                        />
                      </label>
                    </div>
                  </section>
                ) : null}
              </div>
            )}
          </section>

          <section className="card global-panel">
            <div className="panel-head">
              <h2>全局设置</h2>
              <p>Clash 与开机自启属于全局参数，仅需设置一次。</p>
            </div>

            <div className="global-row path-row">
              <label>Clash 可执行路径</label>
              <input
                value={config.global.clash_path}
                onChange={(event) =>
                  setConfig((prev) =>
                    prev ? { ...prev, global: { ...prev.global, clash_path: event.target.value } } : prev
                  )
                }
              />
              <button className="btn ghost" onClick={handleBrowseClashPath} disabled={busy}>
                浏览
              </button>
            </div>

            <div className="global-row port-secret">
              <label>控制端口</label>
              <input
                type="number"
                min={1}
                max={65535}
                value={config.global.clash_port}
                onChange={(event) =>
                  setConfig((prev) =>
                    prev
                      ? {
                          ...prev,
                          global: {
                            ...prev.global,
                            clash_port: Number(event.target.value || 0),
                          },
                        }
                      : prev
                  )
                }
              />
              <label>API 密钥</label>
              <input
                value={config.global.clash_secret}
                onChange={(event) =>
                  setConfig((prev) =>
                    prev ? { ...prev, global: { ...prev.global, clash_secret: event.target.value } } : prev
                  )
                }
              />
            </div>

            <label className="check-line">
              <input
                type="checkbox"
                checked={config.global.enable_app_auto_start}
                onChange={(event) =>
                  setConfig((prev) =>
                    prev
                      ? {
                          ...prev,
                          global: {
                            ...prev.global,
                            enable_app_auto_start: event.target.checked,
                          },
                        }
                      : prev
                  )
                }
              />
              <span>开机自动启动 GameMode Switcher</span>
            </label>

            <div className="runtime-status">{clashRuntimeStatus}</div>

            <div className="global-actions">
              <button className="btn ghost" onClick={handleFetchClashStatus} disabled={busy}>
                获取 Clash 状态
              </button>
              <button className="btn primary" onClick={handleSaveClashSettings} disabled={busy}>
                保存 Clash 设置
              </button>
            </div>
          </section>
        </div>
      </div>

      <div className="status-bar">{status}</div>
    </div>
  );
}

export default App;
