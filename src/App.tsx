import { createSignal } from "solid-js";
import SearchTab from "./tabs/SearchTab";
import CrawlerTab from "./tabs/CrawlerTab";
import ResultsTab from "./tabs/ResultsTab";
import SettingsTab from "./tabs/SettingsTab";
import "./App.css";

type Tab = "search" | "crawler" | "results" | "settings";

export default function App() {
  const [activeTab, setActiveTab] = createSignal<Tab>("search");

  return (
    <div class="min-h-screen bg-base-100">
      {/* Header */}
      <header class="bg-base-200 shadow-md">
        <div class="max-w-7xl mx-auto px-6 py-6">
          <h1 class="text-4xl font-bold text-base-content">📧 Email Scraper</h1>
          <p class="text-sm text-base-content/70 mt-2">
            Search, crawl, and extract email addresses at scale
          </p>
        </div>
      </header>

      {/* Tab Navigation */}
      <div class="bg-base-200 border-b border-base-300">
        <div class="max-w-7xl mx-auto">
          <div class="tabs tabs-bordered">
            <button
              onClick={() => setActiveTab("search")}
              class={`tab ${activeTab() === "search" ? "tab-active" : ""}`}
            >
              🔍 Search
            </button>
            <button
              onClick={() => setActiveTab("crawler")}
              class={`tab ${activeTab() === "crawler" ? "tab-active" : ""}`}
            >
              🕷️ Crawler
            </button>
            <button
              onClick={() => setActiveTab("results")}
              class={`tab ${activeTab() === "results" ? "tab-active" : ""}`}
            >
              📊 Results
            </button>
            <button
              onClick={() => setActiveTab("settings")}
              class={`tab ${activeTab() === "settings" ? "tab-active" : ""}`}
            >
              ⚙️ Settings
            </button>
          </div>
        </div>
      </div>

      {/* Tab Content */}
      <main class="max-w-7xl mx-auto px-6 py-8">
        {activeTab() === "search" && <SearchTab />}
        {activeTab() === "crawler" && <CrawlerTab />}
        {activeTab() === "results" && <ResultsTab />}
        {activeTab() === "settings" && <SettingsTab />}
      </main>
    </div>
  );
}
