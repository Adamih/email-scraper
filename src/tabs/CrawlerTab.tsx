/** @jsxImportSource solid-js */
import { createSignal, For, Show, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface Domain {
  id: string;
  search_query_id: string;
  domain: string;
  url: string;
  title: string | null;
  description: string | null;
  crawl_status: string;
  crawl_started_at: string | null;
  crawl_completed_at: string | null;
  error_message: string | null;
  created_at: string;
}

export default function CrawlerTab() {
  const [pendingDomains, setPendingDomains] = createSignal<Domain[]>([]);
  const [crawling, setCrawling] = createSignal(false);
  const [activeCrawl, setActiveCrawl] = createSignal<string | null>(null);
  const [viewMode, setViewMode] = createSignal<"list" | "table">("table");

  const loadPendingDomains = async () => {
    try {
      const domains = await invoke<Domain[]>("get_pending_domains");
      setPendingDomains(domains);
    } catch (error) {
      console.error("Failed to load pending domains:", error);
    }
  };

  const startCrawl = async (domainId: string) => {
    try {
      setCrawling(true);
      setActiveCrawl(domainId);
      await invoke<string>("start_crawler", { domainId });
      setTimeout(() => {
        loadPendingDomains();
        setCrawling(false);
      }, 2000);
    } catch (error) {
      console.error("Crawl failed:", error);
      alert(`Crawl failed: ${error}`);
      setCrawling(false);
    }
  };

  const startAllCrawls = async () => {
    try {
      setCrawling(true);
      await invoke<string>("start_all_crawlers");
      // Reload domains after a short delay to reflect crawl progress
      setTimeout(() => {
        loadPendingDomains();
      }, 1000);
    } catch (error) {
      console.error("Failed to start all crawls:", error);
      alert(`Failed to start crawls: ${error}`);
      setCrawling(false);
    }
  };

  const removeDomain = async (domainId: string) => {
    try {
      await invoke<string>("delete_domain", { domainId });
      console.log("Domain deleted successfully");
      loadPendingDomains();
    } catch (error) {
      console.error("Failed to delete domain:", error);
      alert(`Failed to delete domain: ${error}`);
    }
  };

  const removeAllPending = async () => {
    if (
      !confirm(
        `Are you sure you want to delete ${
          pendingDomains().filter((d) => d.crawl_status === "pending").length
        } pending domains?`
      )
    ) {
      return;
    }

    try {
      await invoke<string>("delete_all_pending_domains");
      console.log("All pending domains deleted successfully");
      loadPendingDomains();
    } catch (error) {
      console.error("Failed to delete all pending domains:", error);
      alert(`Failed to delete pending domains: ${error}`);
    }
  };

  loadPendingDomains();

  createEffect(() => {
    if (crawling()) {
      const interval = setInterval(loadPendingDomains, 5000);
      return () => clearInterval(interval);
    }
  });

  return (
    <div class="space-y-6">
      {/* Controls */}
      <div class="card bg-base-100 shadow-lg border border-base-200">
        <div class="card-body">
          <h2 class="card-title text-2xl">🕷️ Crawler Controls</h2>

          <div class="flex gap-2 flex-wrap items-center">
            <button
              onClick={() => loadPendingDomains()}
              disabled={crawling()}
              class="btn btn-outline"
            >
              🔄 Refresh
            </button>

            <button
              onClick={() => startAllCrawls()}
              disabled={
                crawling() ||
                pendingDomains().filter((d) => d.crawl_status === "pending")
                  .length === 0
              }
              class="btn btn-success"
            >
              ▶️ Crawl All Pending
            </button>

            <button
              onClick={() => removeAllPending()}
              disabled={
                pendingDomains().filter((d) => d.crawl_status === "pending")
                  .length === 0
              }
              class="btn btn-error"
            >
              🗑️ Remove All Pending
            </button>

            <div class="form-control ml-auto">
              <label class="label py-1 gap-2 cursor-pointer">
                <span class="label-text text-sm">
                  {viewMode() === "table" ? "📊 Table" : "📝 List"}
                </span>
                <input
                  type="checkbox"
                  checked={viewMode() === "list"}
                  onChange={(e) =>
                    setViewMode(e.currentTarget.checked ? "list" : "table")
                  }
                  class="toggle toggle-sm"
                />
              </label>
            </div>
          </div>
        </div>
      </div>

      {/* Pending Domains */}
      <Show when={pendingDomains().length > 0}>
        <div class="card bg-base-100 shadow-lg border border-base-200">
          <div class="card-body">
            <h2 class="card-title text-2xl">
              📌 Domains to Crawl (
              {
                pendingDomains().filter((d) => d.crawl_status === "pending")
                  .length
              }
              )
            </h2>

            {/* Table View */}
            <Show when={viewMode() === "table"}>
              <div class="overflow-x-auto">
                <table class="table table-zebra">
                  <thead>
                    <tr>
                      <th>Domain</th>
                      <th>Title</th>
                      <th>Status</th>
                      <th>Action</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={pendingDomains()}>
                      {(domain) => (
                        <tr class="hover">
                          <td class="font-bold text-primary">
                            {domain.domain}
                          </td>
                          <td class="max-w-xs truncate">
                            {domain.title || "-"}
                          </td>
                          <td>
                            <div class="badge badge-sm">
                              {domain.crawl_status}
                            </div>
                          </td>
                          <td>
                            <Show when={domain.crawl_status === "pending"}>
                              <div class="flex gap-2">
                                <button
                                  onClick={() => startCrawl(domain.id)}
                                  disabled={crawling()}
                                  class="btn btn-xs btn-primary"
                                >
                                  {activeCrawl() === domain.id && crawling() ? (
                                    <>
                                      <span class="loading loading-spinner loading-xs"></span>
                                    </>
                                  ) : (
                                    "Start"
                                  )}
                                </button>
                                <button
                                  onClick={() => removeDomain(domain.id)}
                                  class="btn btn-xs btn-error"
                                >
                                  ✕
                                </button>
                              </div>
                            </Show>
                          </td>
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </div>
            </Show>

            {/* List View */}
            <Show when={viewMode() === "list"}>
              <div class="space-y-3 max-h-96 overflow-y-auto">
                <For each={pendingDomains()}>
                  {(domain) => (
                    <div class="card bg-base-200 border border-base-300 hover:border-base-400">
                      <div class="card-body p-4">
                        <div class="flex items-start justify-between gap-4">
                          <div class="flex-1">
                            <h3 class="font-bold text-lg text-primary">
                              {domain.domain}
                            </h3>
                            <Show when={domain.title}>
                              <p class="text-sm opacity-75 mt-1">
                                {domain.title}
                              </p>
                            </Show>
                            <p class="text-xs opacity-60 mt-2">
                              {new Date(domain.created_at).toLocaleDateString()}
                            </p>
                          </div>

                          <div class="flex items-center gap-2">
                            <div class="badge badge-lg">
                              {domain.crawl_status}
                            </div>

                            <Show when={domain.crawl_status === "pending"}>
                              <button
                                onClick={() => startCrawl(domain.id)}
                                disabled={crawling()}
                                class="btn btn-sm btn-primary"
                              >
                                {activeCrawl() === domain.id && crawling() ? (
                                  <>
                                    <span class="loading loading-spinner loading-xs"></span>
                                    Crawling...
                                  </>
                                ) : (
                                  "Start"
                                )}
                              </button>
                              <button
                                onClick={() => removeDomain(domain.id)}
                                class="btn btn-sm btn-error"
                              >
                                ✕
                              </button>
                            </Show>
                          </div>
                        </div>

                        <Show when={domain.error_message}>
                          <div class="alert alert-error mt-2 py-2">
                            <span class="text-xs">{domain.error_message}</span>
                          </div>
                        </Show>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </div>
      </Show>

      <Show when={pendingDomains().length === 0}>
        <div class="card bg-base-100 shadow-lg border border-base-200">
          <div class="card-body items-center text-center">
            <p class="text-lg text-base-content/60">
              No domains pending crawl.
            </p>
            <p class="text-sm text-base-content/50">
              Execute a search first to discover domains.
            </p>
          </div>
        </div>
      </Show>
    </div>
  );
}
