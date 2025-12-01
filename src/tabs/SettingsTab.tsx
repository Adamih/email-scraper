import { createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface ConfigState {
  google_api_key: string;
  google_search_engine_id: string;
  google_search_results: number;
  max_pages_per_domain: number;
  max_crawl_depth: number;
  request_timeout_secs: number;
  rate_limit_delay_ms: number;
}

export default function SettingsTab() {
  const [config, setConfig] = createSignal<ConfigState>({
    google_api_key: "",
    google_search_engine_id: "",
    google_search_results: 10,
    max_pages_per_domain: 100,
    max_crawl_depth: 3,
    request_timeout_secs: 10,
    rate_limit_delay_ms: 500,
  });
  const [saving, setSaving] = createSignal(false);
  const [saved, setSaved] = createSignal(false);

  const loadConfig = async () => {
    try {
      const cfg = await invoke<ConfigState>("get_config");
      setConfig(cfg);
    } catch (error) {
      console.error("Failed to load config:", error);
      alert(`Failed to load settings: ${error}`);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await invoke("set_config", { config: config() });
      setSaved(true);
      setTimeout(() => setSaved(false), 3000);
    } catch (error) {
      console.error("Failed to save config:", error);
      alert(`Failed to save settings: ${error}`);
    } finally {
      setSaving(false);
    }
  };

  const updateConfig = (key: keyof ConfigState, value: any) => {
    setConfig((prev) => ({ ...prev, [key]: value }));
  };

  loadConfig();

  return (
    <div class="space-y-6">
      {/* Google API Settings */}
      <div class="card bg-base-100 shadow-lg border border-base-200">
        <div class="card-body">
          <h2 class="card-title text-2xl">🔑 Google API Configuration</h2>

          {/* API Key */}
          <div class="form-control">
            <label class="label">
              <span class="label-text font-semibold">Google API Key</span>
            </label>
            <input
              type="password"
              placeholder="Enter your Google API key"
              value={config().google_api_key}
              onInput={(e) =>
                updateConfig("google_api_key", e.currentTarget.value)
              }
              class="input input-bordered"
            />
            <label class="label">
              <span class="label-text-alt">
                Get it from{" "}
                <a
                  href="https://console.cloud.google.com"
                  target="_blank"
                  class="link link-primary"
                >
                  Google Cloud Console
                </a>
              </span>
            </label>
          </div>

          {/* Search Engine ID */}
          <div class="form-control">
            <label class="label">
              <span class="label-text font-semibold">Search Engine ID</span>
            </label>
            <input
              type="text"
              placeholder="Enter your Custom Search Engine ID (cx)"
              value={config().google_search_engine_id}
              onInput={(e) =>
                updateConfig("google_search_engine_id", e.currentTarget.value)
              }
              class="input input-bordered"
            />
            <label class="label">
              <span class="label-text-alt">
                Create a Custom Search Engine at{" "}
                <a
                  href="https://programmablesearchengine.google.com"
                  target="_blank"
                  class="link link-primary"
                >
                  Google Programmable Search Engine
                </a>
              </span>
            </label>
          </div>

          {/* Number of Search Results */}
          <div class="form-control">
            <label class="label">
              <span class="label-text font-semibold">
                Search Results Per Query
              </span>
            </label>
            <input
              type="number"
              min="1"
              max="100"
              value={config().google_search_results}
              onInput={(e) =>
                updateConfig(
                  "google_search_results",
                  parseInt(e.currentTarget.value)
                )
              }
              class="input input-bordered"
            />
            <label class="label">
              <span class="label-text-alt">
                Number of domains to fetch from Google search (1-100, uses
                pagination for results above 10)
              </span>
            </label>
          </div>
        </div>
      </div>

      {/* Crawler Settings */}
      <div class="card bg-base-100 shadow-lg border border-base-200">
        <div class="card-body">
          <h2 class="card-title text-2xl">🕷️ Crawler Configuration</h2>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Max Pages */}
            <div class="form-control">
              <label class="label">
                <span class="label-text font-semibold">
                  Max Pages Per Domain
                </span>
              </label>
              <input
                type="number"
                min="1"
                max="1000"
                value={config().max_pages_per_domain}
                onInput={(e) =>
                  updateConfig(
                    "max_pages_per_domain",
                    parseInt(e.currentTarget.value)
                  )
                }
                class="input input-bordered"
              />
              <label class="label">
                <span class="label-text-alt">
                  Maximum pages to crawl per domain
                </span>
              </label>
            </div>

            {/* Max Depth */}
            <div class="form-control">
              <label class="label">
                <span class="label-text font-semibold">Max Crawl Depth</span>
              </label>
              <input
                type="number"
                min="1"
                max="10"
                value={config().max_crawl_depth}
                onInput={(e) =>
                  updateConfig(
                    "max_crawl_depth",
                    parseInt(e.currentTarget.value)
                  )
                }
                class="input input-bordered"
              />
              <label class="label">
                <span class="label-text-alt">Directory depth to crawl</span>
              </label>
            </div>

            {/* Request Timeout */}
            <div class="form-control">
              <label class="label">
                <span class="label-text font-semibold">
                  Request Timeout (seconds)
                </span>
              </label>
              <input
                type="number"
                min="1"
                max="60"
                value={config().request_timeout_secs}
                onInput={(e) =>
                  updateConfig(
                    "request_timeout_secs",
                    parseInt(e.currentTarget.value)
                  )
                }
                class="input input-bordered"
              />
              <label class="label">
                <span class="label-text-alt">
                  How long to wait for HTTP response
                </span>
              </label>
            </div>

            {/* Rate Limit Delay */}
            <div class="form-control">
              <label class="label">
                <span class="label-text font-semibold">
                  Rate Limit Delay (milliseconds)
                </span>
              </label>
              <input
                type="number"
                min="0"
                max="10000"
                value={config().rate_limit_delay_ms}
                onInput={(e) =>
                  updateConfig(
                    "rate_limit_delay_ms",
                    parseInt(e.currentTarget.value)
                  )
                }
                class="input input-bordered"
              />
              <label class="label">
                <span class="label-text-alt">
                  Delay between requests to be respectful to servers
                </span>
              </label>
            </div>
          </div>
        </div>
      </div>

      {/* Save Button */}
      <div class="card bg-base-100 shadow-lg border border-base-200">
        <div class="card-body">
          <button
            onClick={handleSave}
            disabled={saving()}
            class="btn btn-primary btn-lg"
          >
            {saving() ? (
              <>
                <span class="loading loading-spinner"></span>
                Saving...
              </>
            ) : (
              "💾 Save Settings"
            )}
          </button>

          <Show when={saved()}>
            <div class="alert alert-success mt-4">
              <span>✅ Settings saved successfully!</span>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
}
