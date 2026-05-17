### Known Bugs

1. If GET request has a body, sending the request shows Error:builder error in the response panel.

### Stats Upgrade

- Instead of just showing the Initials of the stats, show the selected stats' name and then the rest of the headers should be just Initials.
- The Stats Main Header should be called "Stats: [Selected Stats Name]" instead of just Selected Stat name. This way, it's clear which stats are being shown without having to look at the tabs. The tabs can still have the initials (O, P, D, M) for quick reference, but the main header should be descriptive.
- There is no network stat. We can add a new tab for Network stats that includes:
  - DNS Lookup Time
  - TCP Handshake Time
  - TLS Handshake Time
  - Time to First Byte (TTFB)
  - Content Download Time
