import { createSignal, For, Show, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

type NotificationStatus = "idle" | "loading" | "success" | "error";

interface Email {
  id: string;
  search_query_id: string;
  domain_id: string;
  domain: string;
  email: string;
  source_url: string;
  page_title: string | null;
  tags: string;
  found_at: string;
}

interface EmailListResponse {
  emails: Email[];
  total: number;
  page: number;
  per_page: number;
}

export default function ResultsTab() {
  const [emails, setEmails] = createSignal<Email[]>([]);
  const [total, setTotal] = createSignal(0);
  const [page, setPage] = createSignal(1);
  const [perPage, setPerPage] = createSignal(50);
  const [domainStats, setDomainStats] = createSignal<Array<[string, number]>>(
    []
  );
  const [filterDomain, setFilterDomain] = createSignal<string | null>(null);
  const [sortBy, setSortBy] = createSignal<string>("found_at_desc");
  const [loading, setLoading] = createSignal(false);
  const [viewMode, setViewMode] = createSignal<"list" | "table">("table");
  const [exportStatus, setExportStatus] =
    createSignal<NotificationStatus>("idle");
  const [exportMessage, setExportMessage] = createSignal<string>("");

  const loadEmails = async (pg: number = 1, domain: string | null = null) => {
    setLoading(true);
    try {
      let response: EmailListResponse;

      if (domain) {
        response = await invoke<EmailListResponse>("get_emails_by_domain", {
          domain,
          page: pg,
          perPage: perPage(),
        });
      } else {
        response = await invoke<EmailListResponse>("get_emails", {
          page: pg,
          perPage: perPage(),
        });
      }

      let sortedEmails = [...response.emails];
      switch (sortBy()) {
        case "email_asc":
          sortedEmails.sort((a, b) => a.email.localeCompare(b.email));
          break;
        case "email_desc":
          sortedEmails.sort((a, b) => b.email.localeCompare(a.email));
          break;
        case "domain_asc":
          sortedEmails.sort((a, b) => a.domain.localeCompare(b.domain));
          break;
        case "domain_desc":
          sortedEmails.sort((a, b) => b.domain.localeCompare(a.domain));
          break;
        case "found_at_asc":
          sortedEmails.sort(
            (a, b) =>
              new Date(a.found_at).getTime() - new Date(b.found_at).getTime()
          );
          break;
        case "found_at_desc":
        default:
          sortedEmails.sort(
            (a, b) =>
              new Date(b.found_at).getTime() - new Date(a.found_at).getTime()
          );
          break;
      }

      setEmails(sortedEmails);
      setTotal(response.total);
      setPage(pg);
    } catch (error) {
      console.error("Failed to load emails:", error);
      alert(`Failed to load emails: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const loadDomainStats = async () => {
    try {
      const stats = await invoke<Array<[string, number]>>(
        "get_domain_statistics"
      );
      setDomainStats(stats);
    } catch (error) {
      console.error("Failed to load domain stats:", error);
    }
  };

  const exportToExcel = async () => {
    try {
      setExportStatus("loading");
      setExportMessage("Exporting emails to CSV...");

      // Invoke the backend export function
      const result = await invoke<string>("export_to_csv");

      setExportStatus("success");
      setExportMessage(`✅ ${result}`);
      console.log("Export successful:", result);

      setTimeout(() => setExportStatus("idle"), 4000);
    } catch (error) {
      console.error("Export failed:", error);
      setExportStatus("error");
      setExportMessage(`❌ Export failed: ${error}`);
      setTimeout(() => setExportStatus("idle"), 6000);
    }
  };

  loadEmails();
  loadDomainStats();

  const interval = setInterval(loadDomainStats, 10000);

  return (
    <div class="space-y-6">
      {/* Export Status Notification */}
      {exportStatus() === "error" && (
        <div class="alert alert-error shadow-lg">
          <div>
            <span>{exportMessage()}</span>
          </div>
        </div>
      )}
      {exportStatus() === "success" && (
        <div class="alert alert-success shadow-lg">
          <div>
            <span>{exportMessage()}</span>
          </div>
        </div>
      )}
      {exportStatus() === "loading" && (
        <div class="alert alert-info shadow-lg">
          <div>
            <span class="loading loading-spinner loading-sm"></span>
            <span>{exportMessage()}</span>
          </div>
        </div>
      )}

      {/* Statistics */}
      <Show when={domainStats().length > 0}>
        <div class="card bg-base-100 shadow-lg border border-base-200">
          <div class="card-body">
            <h2 class="card-title text-2xl">📈 Domain Statistics</h2>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              <For each={domainStats().slice(0, 9)}>
                {(stat) => (
                  <button
                    onClick={() => {
                      setFilterDomain(stat[0]);
                      loadEmails(1, stat[0]);
                    }}
                    class="stat bg-gradient-to-br from-primary/10 to-secondary/10 border border-primary/20 hover:border-primary/40 cursor-pointer transition-all"
                  >
                    <div class="stat-title text-sm">{stat[0]}</div>
                    <div class="stat-value text-2xl text-primary">
                      {stat[1]}
                    </div>
                    <div class="stat-desc text-xs">emails found</div>
                  </button>
                )}
              </For>
            </div>
          </div>
        </div>
      </Show>

      {/* Controls */}
      <div class="card bg-base-100 shadow-lg border border-base-200">
        <div class="card-body">
          <div class="flex items-center justify-between mb-4">
            <h2 class="card-title text-2xl">
              {filterDomain()
                ? `📧 Emails from ${filterDomain()} (${total()})`
                : `📧 All Emails (${total()})`}
            </h2>

            <Show when={filterDomain()}>
              <button
                onClick={() => {
                  setFilterDomain(null);
                  loadEmails(1, null);
                }}
                class="btn btn-sm btn-ghost"
              >
                ✕ Clear Filter
              </button>
            </Show>
          </div>

          <div class="flex gap-4 flex-wrap items-end">
            <div class="form-control">
              <label class="label py-1">
                <span class="label-text">Sort by:</span>
              </label>
              <select
                value={sortBy()}
                onChange={(e) => setSortBy(e.currentTarget.value)}
                class="select select-bordered select-sm"
              >
                <option value="found_at_desc">Newest First</option>
                <option value="found_at_asc">Oldest First</option>
                <option value="email_asc">Email (A-Z)</option>
                <option value="email_desc">Email (Z-A)</option>
                <option value="domain_asc">Domain (A-Z)</option>
                <option value="domain_desc">Domain (Z-A)</option>
              </select>
            </div>

            <div class="form-control">
              <label class="label py-1">
                <span class="label-text">Per page:</span>
              </label>
              <select
                value={perPage().toString()}
                onChange={(e) => {
                  setPerPage(Number(e.currentTarget.value));
                  loadEmails(1, filterDomain());
                }}
                class="select select-bordered select-sm"
              >
                <option value="10">10</option>
                <option value="25">25</option>
                <option value="50">50</option>
                <option value="100">100</option>
              </select>
            </div>

            <div class="form-control">
              <label class="label py-1 gap-2 cursor-pointer">
                <span class="label-text">
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

            <button
              onClick={() => exportToExcel()}
              class="btn btn-success btn-sm ml-auto"
            >
              📥 Export CSV
            </button>
          </div>
        </div>
      </div>

      {/* Email List/Table */}
      <Show
        when={emails().length > 0}
        fallback={
          <div class="card bg-base-100 shadow-lg border border-base-200">
            <div class="card-body items-center text-center">
              <p class="text-lg text-base-content/60">No emails found</p>
            </div>
          </div>
        }
      >
        <div class="card bg-base-100 shadow-lg border border-base-200">
          <div class="card-body p-0">
            {/* Table View */}
            <Show when={viewMode() === "table"}>
              <div class="overflow-x-auto">
                <table class="table table-zebra">
                  <thead>
                    <tr>
                      <th>Email</th>
                      <th>Domain</th>
                      <th>Page Title</th>
                      <th>Found</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={emails()}>
                      {(email) => (
                        <tr class="hover">
                          <td class="font-mono text-primary">
                            <a href={`mailto:${email.email}`} class="link">
                              {email.email}
                            </a>
                          </td>
                          <td>{email.domain}</td>
                          <td class="max-w-xs truncate">
                            {email.page_title || "-"}
                          </td>
                          <td class="text-xs">
                            {new Date(email.found_at).toLocaleDateString()}
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
              <div class="space-y-2 p-4">
                <For each={emails()}>
                  {(email) => (
                    <div class="border border-base-300 rounded-lg p-3 hover:bg-base-200 transition-colors">
                      <div class="flex items-start justify-between gap-2">
                        <div class="flex-1">
                          <div class="font-mono text-primary font-semibold">
                            <a href={`mailto:${email.email}`} class="link">
                              {email.email}
                            </a>
                          </div>
                          <div class="text-sm text-base-content/70 mt-1">
                            <span class="badge badge-sm badge-outline">
                              {email.domain}
                            </span>
                            <span class="ml-2 text-xs">
                              {new Date(email.found_at).toLocaleDateString()}
                            </span>
                          </div>
                          <Show when={email.page_title}>
                            <div class="text-xs text-base-content/60 mt-2 line-clamp-2">
                              📄 {email.page_title}
                            </div>
                          </Show>
                        </div>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            {/* Pagination */}
            <div class="flex items-center justify-between p-4 border-t border-base-200">
              <div class="text-sm">
                Page {page()} of {Math.ceil(total() / perPage())}
              </div>

              <div class="join">
                <button
                  onClick={() => {
                    const prevPage = page() - 1;
                    if (prevPage > 0) loadEmails(prevPage, filterDomain());
                  }}
                  disabled={page() === 1 || loading()}
                  class="join-item btn btn-sm"
                >
                  « Previous
                </button>

                <button
                  onClick={() => {
                    const nextPage = page() + 1;
                    if (nextPage <= Math.ceil(total() / perPage()))
                      loadEmails(nextPage, filterDomain());
                  }}
                  disabled={
                    page() >= Math.ceil(total() / perPage()) || loading()
                  }
                  class="join-item btn btn-sm"
                >
                  Next »
                </button>
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
