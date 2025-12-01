import { createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface SearchQuery {
  id: string;
  query: string;
  tags: string;
  created_at: string;
  completed_at: string | null;
}

interface DomainResult {
  id: string;
  domain: string;
  title: string | null;
  description: string | null;
  crawl_status: string;
}

type NotificationStatus = "idle" | "loading" | "success" | "error";

export default function SearchTab() {
  const [query, setQuery] = createSignal("");
  const [tags, setTags] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [status, setStatus] = createSignal<NotificationStatus>("idle");
  const [statusMessage, setStatusMessage] = createSignal("");
  const [searches, setSearches] = createSignal<SearchQuery[]>([]);
  const [selectedSearchId, setSelectedSearchId] = createSignal<string | null>(
    null
  );
  const [domainResults, setDomainResults] = createSignal<DomainResult[]>([]);
  const [domainViewMode, setDomainViewMode] = createSignal<"list" | "table">(
    "table"
  );
  const [deleteConfirmId, setDeleteConfirmId] = createSignal<string | null>(
    null
  );
  const [deleteConfirmQuery, setDeleteConfirmQuery] = createSignal<string>("");
  const [isDeleting, setIsDeleting] = createSignal(false);

  const handleSearch = async () => {
    if (!query().trim()) {
      setStatus("error");
      setStatusMessage("Please enter a search query");
      setTimeout(() => setStatus("idle"), 4000);
      return;
    }

    setLoading(true);
    setStatus("loading");
    setStatusMessage("Searching...");
    try {
      const tagArray = tags()
        .split(",")
        .map((t: string) => t.trim())
        .filter((t: string) => t.length > 0);

      await invoke<DomainResult[]>("execute_google_search", {
        request: { query: query(), tags: tagArray },
      });

      setStatus("success");
      setStatusMessage(`✅ Search completed! Found results for "${query()}"`);
      setQuery("");
      setTags("");
      loadSearches();
      setTimeout(() => setStatus("idle"), 4000);
    } catch (error: any) {
      console.error("Search failed:", error);
      const errorMsg = error?.message || String(error);
      setStatus("error");
      setStatusMessage(`❌ Search failed: ${errorMsg}`);
      setTimeout(() => setStatus("idle"), 6000);
    } finally {
      setLoading(false);
    }
  };

  const loadSearches = async () => {
    try {
      const searches = await invoke<SearchQuery[]>("get_searches");
      setSearches(searches);
    } catch (error) {
      console.error("Failed to load searches:", error);
    }
  };

  const handleSelectSearch = async (searchId: string) => {
    setSelectedSearchId(searchId);
    try {
      const domains = await invoke<DomainResult[]>("get_domains_for_search", {
        searchId,
      });
      setDomainResults(domains);
    } catch (error) {
      console.error("Failed to load domains:", error);
      setStatus("error");
      setStatusMessage(`Failed to load domains: ${error}`);
      setTimeout(() => setStatus("idle"), 4000);
    }
  };

  const handleDeleteSearch = (searchId: string, searchQuery: string) => {
    console.log(
      "Opening delete confirmation modal for:",
      searchId,
      searchQuery
    );
    setDeleteConfirmId(searchId);
    setDeleteConfirmQuery(searchQuery);
  };

  const confirmDelete = async () => {
    const searchId = deleteConfirmId();
    if (!searchId) return;

    try {
      console.log("Invoking delete_search command with searchId:", searchId);
      setIsDeleting(true);
      setStatus("loading");
      setStatusMessage("Deleting search...");
      const result = await invoke<string>("delete_search", {
        searchId: searchId,
      });
      console.log("Delete successful! Response:", result);
      setStatus("success");
      setStatusMessage("✅ Search deleted successfully");
      setSelectedSearchId(null);
      setDomainResults([]);
      await loadSearches();
      setDeleteConfirmId(null);
      setDeleteConfirmQuery("");
      setTimeout(() => setStatus("idle"), 3000);
    } catch (error: any) {
      console.error("Delete failed - Full error:", error);
      console.error("Error message:", error?.message);
      const errorMsg = error?.message || String(error);
      setStatus("error");
      setStatusMessage(`❌ Delete failed: ${errorMsg}`);
      setTimeout(() => setStatus("idle"), 6000);
    } finally {
      setIsDeleting(false);
    }
  };

  const cancelDelete = () => {
    console.log("User canceled deletion");
    setDeleteConfirmId(null);
    setDeleteConfirmQuery("");
  };

  loadSearches();

  return (
    <div class="space-y-6">
      {/* Status Notification */}
      <div class="alert alert-info shadow-lg">
        <div>
          <span>
            ⚙️ Remember to configure your Google API key in the{" "}
            <strong>Settings</strong> tab before searching
          </span>
        </div>
      </div>

      {status() === "error" && (
        <div class="alert alert-error shadow-lg">
          <div>
            <span>{statusMessage()}</span>
          </div>
        </div>
      )}
      {status() === "success" && (
        <div class="alert alert-success shadow-lg">
          <div>
            <span>{statusMessage()}</span>
          </div>
        </div>
      )}
      {status() === "loading" && (
        <div class="alert alert-info shadow-lg">
          <div>
            <span class="loading loading-spinner loading-sm"></span>
            <span>{statusMessage()}</span>
          </div>
        </div>
      )}

      {/* Search Form */}
      <div class="card bg-base-100 shadow-lg border border-base-200">
        <div class="card-body">
          <h2 class="card-title text-2xl">🔍 Execute Google Search</h2>

          <div class="space-y-4">
            <div class="form-control">
              <label class="label">
                <span class="label-text font-semibold">Search Query</span>
              </label>
              <input
                type="text"
                value={query()}
                onInput={(e) => setQuery(e.currentTarget.value)}
                onKeyPress={(e) => {
                  if (e.key === "Enter") handleSearch();
                }}
                placeholder="e.g., web development companies in london"
                class="input input-bordered input-primary focus:outline-none w-full"
              />
            </div>

            <div class="form-control">
              <label class="label">
                <span class="label-text font-semibold">
                  Tags (comma-separated)
                </span>
              </label>
              <input
                type="text"
                value={tags()}
                onInput={(e) => setTags(e.currentTarget.value)}
                placeholder="e.g., startups, tech, london"
                class="input input-bordered input-primary focus:outline-none w-full"
              />
            </div>

            <button
              onClick={handleSearch}
              disabled={loading()}
              class="btn btn-primary btn-block"
            >
              {loading() && (
                <span class="loading loading-spinner loading-sm"></span>
              )}
              {loading() ? "Searching..." : "🔍 Search"}
            </button>
          </div>
        </div>
      </div>

      {/* Recent Searches */}
      <Show when={searches().length > 0}>
        <div class="card bg-base-100 shadow-lg border border-base-200">
          <div class="card-body">
            <h2 class="card-title text-2xl mb-4">📋 Recent Searches</h2>

            <div class="divider my-0"></div>

            <ul class="menu menu-compact w-full space-y-2">
              <For each={searches()}>
                {(search) => {
                  const isSelected = selectedSearchId() === search.id;
                  const parsedTags = (() => {
                    try {
                      return JSON.parse(search.tags);
                    } catch {
                      return [];
                    }
                  })();
                  const formattedDate = new Date(
                    search.created_at
                  ).toLocaleDateString("en-US", {
                    year: "numeric",
                    month: "short",
                    day: "numeric",
                  });

                  return (
                    <li>
                      <button
                        onClick={() => handleSelectSearch(search.id)}
                        class={`rounded-lg transition-all flex-1 justify-between ${
                          isSelected
                            ? "bg-primary text-primary-content shadow-md"
                            : "bg-base-200 hover:bg-base-300"
                        }`}
                      >
                        <div class="flex flex-col items-start flex-1 gap-2 py-2">
                          <div class="flex items-center gap-2 w-full">
                            <div class="font-bold text-base flex-1 truncate">
                              {search.query}
                            </div>
                            <span class="text-xs opacity-70 whitespace-nowrap">
                              {formattedDate}
                            </span>
                          </div>
                          {parsedTags.length > 0 && (
                            <div class="flex flex-wrap gap-1">
                              <For each={parsedTags}>
                                {(tag) => (
                                  <span
                                    class={`badge badge-sm ${
                                      isSelected
                                        ? "badge-primary-content"
                                        : "badge-neutral"
                                    }`}
                                  >
                                    {tag}
                                  </span>
                                )}
                              </For>
                            </div>
                          )}
                        </div>
                      </button>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteSearch(search.id, search.query);
                        }}
                        class={`btn btn-circle btn-xs ${
                          isSelected
                            ? "text-primary-content hover:bg-primary/80"
                            : "hover:bg-base-300"
                        }`}
                        title="Delete this search"
                      >
                        🗑️
                      </button>
                    </li>
                  );
                }}
              </For>
            </ul>
          </div>
        </div>
      </Show>

      {/* Selected Search Results */}
      <Show when={selectedSearchId() && domainResults().length > 0}>
        <div class="card bg-base-100 shadow-lg border border-base-200">
          <div class="card-body">
            <div class="flex items-center justify-between gap-4 mb-4">
              <h2 class="card-title text-2xl">
                🌐 Discovered Domains ({domainResults().length})
              </h2>
              <div class="form-control">
                <label class="label py-1 gap-2 cursor-pointer">
                  <span class="label-text text-sm">
                    {domainViewMode() === "table" ? "📊 Table" : "📝 List"}
                  </span>
                  <input
                    type="checkbox"
                    checked={domainViewMode() === "list"}
                    onChange={(e) =>
                      setDomainViewMode(
                        e.currentTarget.checked ? "list" : "table"
                      )
                    }
                    class="toggle toggle-sm"
                  />
                </label>
              </div>
            </div>

            {/* Table View */}
            <Show when={domainViewMode() === "table"}>
              <div class="overflow-x-auto">
                <table class="table table-zebra">
                  <thead>
                    <tr>
                      <th>Domain</th>
                      <th>Title</th>
                      <th>Status</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={domainResults()}>
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
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </div>
            </Show>

            {/* List View */}
            <Show when={domainViewMode() === "list"}>
              <div class="space-y-2 max-h-96 overflow-y-auto">
                <For each={domainResults()}>
                  {(domain) => (
                    <div class="card bg-base-200 border border-base-300">
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
                          </div>
                          <div class="badge badge-lg">
                            {domain.crawl_status}
                          </div>
                        </div>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </div>
      </Show>

      {/* Delete Confirmation Modal */}
      <Show when={deleteConfirmId() !== null}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div class="card bg-base-100 shadow-2xl border border-base-200 max-w-md">
            <div class="card-body">
              <h2 class="card-title text-2xl">🗑️ Delete Search</h2>
              <p class="text-base-content/80 mt-2">
                Are you sure you want to delete this search?
              </p>
              <p class="text-lg font-semibold text-primary mt-2">
                "{deleteConfirmQuery()}"
              </p>
              <div class="alert alert-warning mt-4 py-2">
                <span class="text-sm">
                  This will permanently remove:
                  <ul class="list-disc list-inside mt-1 text-sm">
                    <li>All discovered domains</li>
                    <li>All extracted emails</li>
                  </ul>
                </span>
              </div>
              <p class="text-sm text-error mt-4">
                This action cannot be undone.
              </p>

              <div class="card-actions justify-end mt-6 gap-3">
                <button
                  onClick={cancelDelete}
                  disabled={isDeleting()}
                  class="btn btn-outline"
                >
                  Cancel
                </button>
                <button
                  onClick={confirmDelete}
                  disabled={isDeleting()}
                  class="btn btn-error"
                >
                  {isDeleting() && (
                    <span class="loading loading-spinner loading-sm"></span>
                  )}
                  {isDeleting() ? "Deleting..." : "Delete Search"}
                </button>
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
