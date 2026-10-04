import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  AppConfig,
  ConfigResponse,
  ExportSettings,
  Factors,
  ProcessedData,
  RawAmounts,
  RoastTotals,
} from "./types";

const DEFAULT_SETTINGS: ExportSettings = {
  factors: { white: 1.2, medium: 1.15, dark: 1.1, decaf: 1.125 },
  roundingIncrement: 0,
  summaryPosition: "footer",
};

const SETTINGS_KEY = "coffee-order-processor-settings-v1";
const STANDARD_ROUNDING = [0, 0.1, 0.25, 0.5, 1];

function savedSettings(): ExportSettings {
  try {
    const value = localStorage.getItem(SETTINGS_KEY);
    if (!value) return DEFAULT_SETTINGS;
    const parsed = JSON.parse(value) as Partial<ExportSettings>;
    return {
      factors: { ...DEFAULT_SETTINGS.factors, ...parsed.factors },
      roundingIncrement:
        typeof parsed.roundingIncrement === "number" ? parsed.roundingIncrement : 0,
      summaryPosition: parsed.summaryPosition === "header" ? "header" : "footer",
    };
  } catch {
    return DEFAULT_SETTINGS;
  }
}

function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

function errorMessage(error: unknown): string {
  return typeof error === "string" ? error : error instanceof Error ? error.message : String(error);
}

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [configPath, setConfigPath] = useState("");
  const [configWarning, setConfigWarning] = useState("");
  const [data, setData] = useState<ProcessedData | null>(null);
  const [settings, setSettings] = useState<ExportSettings>(savedSettings);
  const [raw, setRaw] = useState<RawAmounts | null>(null);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    invoke<ConfigResponse>("load_config")
      .then((response) => {
        setConfig(response.config);
        setConfigPath(response.configPath);
        setConfigWarning(response.warning ?? "");
      })
      .catch((reason) => setError(errorMessage(reason)));
  }, []);

  useEffect(() => {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
    if (!data) {
      setRaw(null);
      return;
    }
    let current = true;
    invoke<RawAmounts>("calculate_requirements", { totals: data.totals, settings })
      .then((result) => {
        if (current) {
          setRaw(result);
          setError("");
        }
      })
      .catch((reason) => current && setError(errorMessage(reason)));
    return () => {
      current = false;
    };
  }, [data, settings]);

  const roundingMode = useMemo(
    () =>
      STANDARD_ROUNDING.includes(settings.roundingIncrement)
        ? String(settings.roundingIncrement)
        : "custom",
    [settings.roundingIncrement],
  );

  async function importCsv() {
    if (!config) return;
    const selected = await open({
      multiple: false,
      filters: [{ name: "CSV order export", extensions: ["csv"] }],
    });
    if (typeof selected !== "string") return;

    setBusy(true);
    setError("");
    setSuccess("");
    try {
      const result = await invoke<ProcessedData>("process_csv", {
        path: selected,
        config,
      });
      setData(result);
    } catch (reason) {
      setData(null);
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  async function exportExcel() {
    if (!config || !data) return;
    const defaultName = fileName(data.sourcePath).replace(/\.csv$/i, "") + "-processed.xlsx";
    const selected = await save({
      defaultPath: defaultName,
      filters: [{ name: "Excel workbook", extensions: ["xlsx"] }],
    });
    if (!selected) return;

    setBusy(true);
    setError("");
    setSuccess("");
    try {
      await invoke("export_excel", { path: selected, data, settings, config });
      setSuccess(`Excel workbook saved to ${selected}`);
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  function updateFactor(key: keyof Factors, value: string) {
    setSettings((current) => ({
      ...current,
      factors: { ...current.factors, [key]: Number(value) },
    }));
  }

  function changeRounding(value: string) {
    setSettings((current) => ({
      ...current,
      roundingIncrement: value === "custom" ? 0.1 : Number(value),
    }));
  }

  function resetDefaults() {
    setSettings(DEFAULT_SETTINGS);
  }

  if (!config) {
    return (
      <main className="loading">
        <div className="spinner" />
        <p>{error || "Loading application settings…"}</p>
      </main>
    );
  }

  const labels = config.labels;
  const categoryRows: Array<[keyof RoastTotals, string]> = [
    ["white", labels.white],
    ["medium", labels.medium],
    ["dark", labels.dark],
    ["espresso", labels.espresso],
    ["decaf", labels.decaf],
    ["other", labels.other],
  ];
  const totalBags = data
    ? Object.values(data.totals).reduce((sum, quantity) => sum + quantity, 0)
    : 0;

  return (
    <main className="app-shell">
      <header className="hero">
        <div>
          <p className="eyebrow">Production worksheet</p>
          <h1>{labels.appTitle}</h1>
          <p>Turn a Shopify order export into a clean roasting and shipping workbook.</p>
        </div>
        <div className="hero-actions">
          <button className="button primary" onClick={importCsv} disabled={busy}>
            {labels.importButton}
          </button>
          <button className="button secondary" onClick={exportExcel} disabled={busy || !data}>
            {labels.exportButton}
          </button>
          <button
            className="button ghost"
            onClick={() => window.print()}
            disabled={busy || !data || !raw}
          >
            Print Summary
          </button>
        </div>
      </header>

      {error && <div className="notice error">{error}</div>}
      {success && <div className="notice success">{success}</div>}
      {configWarning && <div className="notice warning">{configWarning}</div>}

      <section className="workspace">
        <aside className="panel settings-panel">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">Calculation setup</p>
              <h2>Raw coffee ratios</h2>
            </div>
            <button className="text-button" onClick={resetDefaults}>Reset defaults</button>
          </div>

          <div className="factor-grid">
            {(["white", "medium", "dark", "decaf"] as const).map((key) => (
              <label key={key}>
                <span>{key === "dark" ? `${labels.dark} + ${labels.espresso}` : labels[key]}</span>
                <input
                  type="number"
                  min="0.001"
                  step="0.001"
                  value={settings.factors[key]}
                  onChange={(event) => updateFactor(key, event.target.value)}
                />
              </label>
            ))}
          </div>

          <label className="field">
            <span>Round each raw roast amount up to</span>
            <select value={roundingMode} onChange={(event) => changeRounding(event.target.value)}>
              <option value="0">No rounding</option>
              <option value="0.1">Next 0.1 lb</option>
              <option value="0.25">Next 0.25 lb</option>
              <option value="0.5">Next 0.5 lb</option>
              <option value="1">Next whole lb</option>
              <option value="custom">Custom increment</option>
            </select>
          </label>
          {roundingMode === "custom" && (
            <label className="field">
              <span>Custom increment (lb)</span>
              <input
                type="number"
                min="0.001"
                step="0.001"
                value={settings.roundingIncrement}
                onChange={(event) =>
                  setSettings((current) => ({
                    ...current,
                    roundingIncrement: Number(event.target.value),
                  }))
                }
              />
            </label>
          )}

          <fieldset>
            <legend>Excel summary position</legend>
            <label className="radio">
              <input
                type="radio"
                checked={settings.summaryPosition === "footer"}
                onChange={() =>
                  setSettings((current) => ({ ...current, summaryPosition: "footer" }))
                }
              />
              Footer (default)
            </label>
            <label className="radio">
              <input
                type="radio"
                checked={settings.summaryPosition === "header"}
                onChange={() =>
                  setSettings((current) => ({ ...current, summaryPosition: "header" }))
                }
              />
              Header
            </label>
          </fieldset>

          <details className="config-note">
            <summary>Editable labels file</summary>
            <p>Change product mappings and display labels in:</p>
            <code>{configPath}</code>
            <p>Restart the app after editing the file.</p>
          </details>
        </aside>

        <section className="content-column">
          {!data ? (
            <div className="panel empty-state">
              <div className="file-icon">CSV</div>
              <h2>Import an order export</h2>
              <p>Select a Shopify CSV to calculate bag totals and raw coffee requirements.</p>
              <button className="button primary" onClick={importCsv} disabled={busy}>
                Choose CSV
              </button>
            </div>
          ) : (
            <>
              <section className="panel file-summary">
                <div>
                  <p className="eyebrow">Imported file</p>
                  <h2>{fileName(data.sourcePath)}</h2>
                  <p>{data.customers.length} customers · {data.lineItemCount} line items</p>
                </div>
                <button className="button ghost no-print" onClick={importCsv} disabled={busy}>
                  Replace file
                </button>
              </section>

              <section className="stats-grid">
                {categoryRows.map(([key, label]) => (
                  <article className={`stat-card ${key === "other" ? "muted" : ""}`} key={key}>
                    <span>{label}</span>
                    <strong>{data.totals[key]}</strong>
                    <small>bags</small>
                  </article>
                ))}
                <article className="stat-card total">
                  <span>{labels.totalBags}</span>
                  <strong>{totalBags}</strong>
                  <small>bags</small>
                </article>
              </section>

              {raw && (
                <section className="panel">
                  <div className="panel-heading">
                    <div>
                      <p className="eyebrow">Production estimate</p>
                      <h2>{labels.rawAmounts}</h2>
                    </div>
                    <strong className="grand-total">
                      {labels.rawTotal}: {raw.total.toFixed(2)} lb
                    </strong>
                  </div>
                  <div className="raw-grid">
                    <div><span>{labels.rawWhite}</span><strong>{raw.white.toFixed(2)} lb</strong></div>
                    <div><span>{labels.rawMedium}</span><strong>{raw.medium.toFixed(2)} lb</strong></div>
                    <div><span>{labels.rawDark}</span><strong>{raw.dark.toFixed(2)} lb</strong></div>
                    <div><span>{labels.rawDecaf}</span><strong>{raw.decaf.toFixed(2)} lb</strong></div>
                  </div>
                </section>
              )}

              {(data.unmatchedProducts.length > 0 || data.warnings.length > 0) && (
                <section className="notice warning">
                  <strong>Review before exporting</strong>
                  {data.unmatchedProducts.length > 0 && (
                    <p>
                      Unmapped products are counted as {labels.other}:{" "}
                      {data.unmatchedProducts.join(", ")}
                    </p>
                  )}
                  {data.warnings.map((warning) => <p key={warning}>{warning}</p>)}
                </section>
              )}

              <section className="panel table-panel">
                <div className="panel-heading">
                  <div>
                    <p className="eyebrow">Excel preview</p>
                    <h2>Customer bag totals</h2>
                  </div>
                </div>
                <div className="table-wrap">
                  <table>
                    <thead>
                      <tr>
                        <th>{labels.shippingName}</th>
                        {categoryRows.map(([key, label]) => <th key={key}>{label}</th>)}
                      </tr>
                    </thead>
                    <tbody>
                      {data.customers.map((customer) => (
                        <tr key={customer.shippingName}>
                          <td>{customer.shippingName}</td>
                          {categoryRows.map(([key]) => <td key={key}>{customer.totals[key]}</td>)}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </section>
            </>
          )}
        </section>
      </section>
    </main>
  );
}

export default App;
