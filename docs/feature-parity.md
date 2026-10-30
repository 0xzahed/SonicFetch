# IDM feature parity target

SonicFetch targets comparable user-facing download-manager capabilities on Linux. This means implementing independent, interoperable features; it does not mean copying IDM source code, interface assets, branding, or Windows-only integration behavior.

The feature list below is based on IDM's public feature and support pages. Status describes SonicFetch today, not a promise that parity already exists.

| Capability | SonicFetch status | Implementation target |
| --- | --- | --- |
| HTTP and HTTPS downloads, redirects, resume, error recovery | Partial | Finish robust retry, resume, content validation, and progress reporting |
| Dynamic multi-connection segmentation | Missing | Discover range support, split work dynamically, validate each segment, fall back safely |
| FTP and MMS protocols | Missing | Add protocol support where Linux libraries and server behavior allow it |
| Persistent queues and scheduler | Partial | Multiple named queues, start/stop scheduling, retry policy, and orderly shutdown |
| Browser integration and download capture | Missing | Opt-in extensions for supported Linux browsers; show intercepted URLs before adding |
| Media download panel and download-all | Missing | Browser extension surfaces links/media candidates and lets the user select items |
| Site Grabber | Missing | User-configured domain scope, link filters, depth limits, and scheduled mirror jobs |
| Categories, file-type rules, and save folders | Missing | Editable categories and automatic rules with user override |
| Proxy and server authentication | Missing | Proxy settings and supported HTTP auth; never persist secrets in plaintext |
| Speed limits and quotas | Missing | Per-download and queue limits with time-window scheduling |
| Clipboard monitoring and drag-and-drop | Missing | Optional clipboard capture and URL/file drag-and-drop |
| Appearance and languages | Missing | Light/dark theme, configurable columns, translations |
| Antivirus integration | Missing | Optional configured scanner after completion; never launch the downloaded file itself |
| Update checks | Missing | Optional signed update metadata and user-controlled installation |

## Delivery order

1. Complete the core queue: multiple active downloads with a configurable concurrency limit, persisted retries, queue controls, and reliable restart recovery.
2. Add dynamic segmentation and protocol coverage, retaining single-stream fallback for servers that do not support ranges.
3. Build the GTK4/libadwaita main window, add dialog, categories, settings, and progress/details views.
4. Add scheduling, speed/quota limits, and configurable proxy/auth support.
5. Add opt-in browser extensions, media/link selection, clipboard, and drag-and-drop integration.
6. Add the scoped Site Grabber and optional scanner/update integrations after their security and consent flows are designed.
7. Finish translations, packaging, signed updates, and release validation.

Browser capture, clipboard monitoring, site crawling, scanner launching, and update installation must be visible, configurable, and disabled until the user enables them. These features are not telemetry and must not collect unrelated browsing activity.

## Current implementation

The current release is a single-download CLI foundation with SQLite persistence, cross-process pause/cancel, validator-aware resume, retry, and safe file publication. It is not yet feature-equivalent to IDM. See [the implementation roadmap](implementation-roadmap.md) for work completed and remaining.

## References

- [IDM public feature list](https://internetdownloadmanager.com/features2.html)
- [IDM main dialog and queue controls](https://www.internetdownloadmanager.com/support/using_idm/using_idm.html)
- [IDM options and browser integration](https://test.internetdownloadmanager.com/support/options.html)
