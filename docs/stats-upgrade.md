# Stat Upgrade Document

To provide a great developer experience (DX), you should group your response stats into four main categories: **The Essentials, Performance, Security/Auth, and Data Payload.**

Keep the Stats as it is, but add a short cut to change the stats. it can be like how we change body and Auth types with t. show in the stats section at the top which section is being shown, like, Overview, Performance, Security, Payload. and then show the stats accordingly.

---

## 1. The Essentials (Must-Haves)

These should be front and center. They tell the developer instantly whether the request succeeded or failed.

- **Status Code & Phrase:** (e.g., `200 OK`, `404 Not Found`, `500 Internal Server Error`). Color-code these (green for 2xx, orange for 4xx, red for 5xx) for instant visual feedback.
- **HTTP Version:** (e.g., `HTTP/1.1`, `HTTP/2`, `HTTP/3`). Crucial for debugging protocol-specific issues or optimization.

This is the current Stats section. you can keep it same just add a url and method to it.

---

## 2. Performance & Timing Stats

Developers love performance metrics. When a request is slow, they need to know _why_. Providing a breakdown of the network waterfall is incredibly valuable.

- **Total Duration:** The total time from the moment the trigger was pressed to the last byte received.
- **Timing Breakdown (The Network Waterfall):**
- **DNS Lookup:** Time taken to resolve the hostname.
- **TCP Handshake:** Time taken to establish the connection.
- **TLS/SSL Handshake:** Time spent negotiating encryption.
- **Time to First Byte (TTFB):** How long the server took to process the request and start sending data back.
- **Content Download:** Time spent transferring the response payload.

---

## 3. Data & Payload Size

Large payloads degrade performance. Developers need to monitor how much data is moving back and forth, especially for mobile app APIs.

- **Response Size:** Show both the compressed (over-the-wire) size and the uncompressed (actual data) size.
- _Example:_ `Size: 4.2 KB (12.8 KB uncompressed)`

- **Header vs. Body Size:** Break down how much of that data was just metadata (headers) versus the actual content (body).

---

## 4. Metadata & Security (Advanced)

These are invaluable for API rate-limiting, caching, and security audits.

- **Rate Limit Status:** If the API returns standard rate-limiting headers, parse them and show them clearly.
- _Remaining:_ How many requests are left in the current window.
- _Reset Time:_ When the quota refreshes.

- **Cache Status:** Did the response come from the server or a cache? Look for headers like `X-Cache: HIT/MISS` or `Age`.
- **SSL/TLS Certificate Info:** A small lock icon that expands to show the certificate issuer, expiration date, and cipher suite used.

---

# Visual Suggestions

These are just suggestions to keep in mind when designing the UI. You don't have to implement all of these, but they can help make the stats more digestible and visually appealing. So take these only for UI inspiration, not as strict requirements.

---

## 1. The "Overview"

Keep as it is, just add a URL and method to it.

---

## 2. The Timeline / Network Waterfall (Using Sparklines or Blocks)

you can use horizontal block characters (`█`, `▌`, `░`) to create a text-based breakdown of where the time was spent.

```text
┌─ Network Latency  ──────────────────────────────────────────────────┐
│ Total Time: 185ms                                                   │
│                                                                     │
│ DNS Lookup   : ██ 12ms (6%)                                         │
│ TCP Connect  : █ 8ms (4%)                                           │
│ TLS Handshake: █░ 15ms (8%)                                         │
│ TTFB (Server): ██████████████████████████████ 145ms (78%)           │
│ Download     : █ 5ms (4%)                                           │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. Metadata & Rate Limits (Side Panel / Key-Value Wall)

- **Rate Limits:** Display as a mini progress bar if the headers exist.
- `Rate Limit: [██████░░░░] 60/100 (Resets in 14m)`

- **Security:** A simple `🔒 TLSv1.3 (ECDHE-RSA-AES128-GCM-SHA256)` string in a dimmed style.
- **Caching:** `Cache: HIT (Age: 3600s)`

---

## 2. Performance & Latency Category

This is where you break down the network waterfall. Developers need to see _why_ a request was slow at a glance.

- **Data to include:** Total Time, DNS Lookup, TCP Connect, TLS Handshake, TTFB (Time to First Byte), Content Download.
- **Ratatui Widget:** `Gauge` (stacked vertically) or a `Paragraph` utilizing colored block characters (`█`, `░`).
- **Visual Style:** Create a horizontal text-based bar chart. Assign a distinct color to each network phase.

```text
┌─ Network Latency Breakdown ─────────────────────────────────────────┐
│ Total Time: 185ms                                                   │
│                                                                     │
│ DNS Lookup   : ██ 12ms (6%)                                         │
│ TCP Connect  : █ 8ms (4%)                                           │
│ TLS Handshake: █░ 15ms (8%)                                         │
│ TTFB (Server): ██████████████████████████████ 145ms (78%)           │
│ Download     : █ 5ms (4%)                                           │
└─────────────────────────────────────────────────────────────────────┘

```

---

## 3. Payload & Size Category

This tracks data efficiency and weight. It's usually placed right alongside performance stats.

- **Data to include:** Total Size, Body Size, Header Size, Compression Ratio (if `gzip`/`brotli` was used).
- **Ratatui Widget:** `Paragraph` with a two-column key-value layout, or a `BarChart` split between Header vs. Body.
- **Visual Style:** Clean, dimmed labels with bright, highlighted values.

```text
┌─ Payload Size ──────────────────────────────────────────────────────┐
│ Total Data :  14.2 KB                                               │
│ ├── Headers:   1.4 KB  [█░░░░░░░░░] 10%                             │
│ └── Body   :  12.8 KB  [█████████░] 90%                             │
│                                                                     │
│ Compression:  GZIP (Saved 65% over wire)                            │
└─────────────────────────────────────────────────────────────────────┘

```

---

## 4. API Governance Category (The Sidebar)

This tracks rate limiting, caching behavior, and server metadata. It is best suited for a narrow vertical sidebar pane.

- **Data to include:** Rate Limit Remaining, Rate Limit Reset Time, Cache Status (`HIT`/`MISS`/`BYPASS`), Server Type.
- **Ratatui Widget:** `List` or a `Table` with thin columns.
- **Visual Style:** Use a mini text progress bar for the rate limit that turns yellow or red as the user approaches 0% remaining.

```text
┌─ API Context ──┐
│ Caching        │
│ ↳ HIT (Age: 6s)│
│                │
│ Rate Limit     │
│ ↳ 42 / 100     │
│ [████░░░░░░]   │
│                │
│ Server         │
│ ↳ cloudflare   │
└────────────────┘

```
