# Email Scraper 📧

A desktop application for discovering and extracting email addresses from websites at scale. Built with Tauri, Solid.js, Rust, and daisyUI.

## Features

✅ **Google Search Integration** - Search for domains based on queries with custom tags
✅ **Web Crawling** - Automatically discover and crawl websites, extract sitemap URLs
✅ **Email Extraction** - Advanced regex-based email detection with validation
✅ **Database Management** - SQLite database with email organization by query, domain, and tags
✅ **Filtering & Sorting** - Sort emails by date, domain, or address; filter by tags
✅ **Pagination** - View large email datasets with configurable page sizes
✅ **Export to CSV** - Download collected emails as CSV for Excel/spreadsheet use
✅ **Cross-Platform** - Runs on macOS and Windows
✅ **Beautiful UI** - Modern interface with daisyUI components

## Tech Stack

- **Frontend**: Solid.js + TypeScript + daisyUI + Tailwind CSS
- **Desktop Framework**: Tauri 2.x
- **Backend**: Rust + Tokio (async runtime)
- **Database**: SQLite with SQLx for type-safe queries
- **HTTP Client**: Reqwest for web scraping

## Prerequisites

- Node.js 18+ or Bun
- Rust 1.70+
- macOS or Windows

## Installation

1. **Clone the repository**
   ```bash
   cd email-scraper
   ```

2. **Install dependencies**
   ```bash
   bun install
   ```

3. **Set up Google API credentials**
   - Go to [Google Cloud Console](https://console.cloud.google.com)
   - Create a new project
   - Enable the Custom Search API
   - Create an API key
   - Create a Custom Search Engine and get the search engine ID
   - These will be entered in the app's settings tab

## Running the Application

### Development Mode
```bash
bun run tauri dev
```

This will:
- Start the Vite dev server (HMR enabled)
- Compile and launch the Rust backend
- Open the app in a development window

### Production Build
```bash
bun run tauri build
```

This creates platform-specific installers in `src-tauri/target/release/bundle/`

## Usage

### 1. Search Phase (🔍 Search Tab)
1. Enter a search query (e.g., "web development agencies in New York")
2. (Optional) Add tags to organize results (e.g., "agencies, tech, NYC")
3. Click **Search**
4. The app will fetch search results and store discovered domains

### 2. Crawling Phase (🕷️ Crawler Tab)
1. See pending domains from your searches
2. Click **Start** on individual domains, or **Crawl All Pending** to start batch crawling
3. The crawler will:
   - Discover sitemaps
   - Crawl pages with rate limiting
   - Extract emails using regex validation
   - Store results with metadata

### 3. Results Phase (📊 Results Tab)
1. View **Domain Statistics** - click a domain to filter emails from it
2. **Sort and Filter** emails by date, address, or domain
3. **Export to CSV** for use in Excel/spreadsheet applications

## Configuration

Settings can be managed in the Settings tab. Configure:

- **Google API Key** - For Google Custom Search
- **Search Engine ID** - For Custom Search Engine  
- **Max Pages Per Domain** - Crawl limit (default: 100)
- **Max Crawl Depth** - Directory depth limit (default: 3)
- **Request Timeout** - HTTP timeout in seconds (default: 10)
- **Rate Limit Delay** - Milliseconds between requests (default: 500)

## Architecture

### Frontend (`src/`)
- **App.tsx** - Main app with daisyUI tab navigation
- **tabs/SearchTab.tsx** - Google search interface
- **tabs/CrawlerTab.tsx** - Crawling control panel
- **tabs/ResultsTab.tsx** - Email results with filtering/export

### Backend (`src-tauri/src/`)
- **models.rs** - Data structures
- **db.rs** - SQLite operations
- **email_extractor.rs** - Email regex extraction
- **commands/** - Tauri command handlers
  - **search.rs** - Google Custom Search API
  - **crawler.rs** - Website crawling
  - **emails.rs** - Email queries
  - **config.rs** - Settings management

## Database

SQLite database with tables for:
- `search_queries` - Search history with tags
- `domains` - Discovered domains with crawl status
- `emails` - Extracted emails with metadata
- `config` - Application settings

## Troubleshooting

### "Google API key not configured"
Enter your credentials in the Settings tab.

### Crawling is slow
Adjust the **Rate Limit Delay** and **Max Pages Per Domain** settings.

### Some emails not found
The regex filters out common false positives. Check the original page source.

## IDE Setup

Recommended:
- [VS Code](https://code.visualstudio.com/)
- [Tauri Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT
