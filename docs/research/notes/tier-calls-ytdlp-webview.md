# Tier calls made while profiling MoneyPrinterTurbo / Hypit / xhs-account-teardown / Kazumi (2026-10-30)

Data for sibling research agents; adjust if the group settles differently.

- yt-dlp wrappers (Hypit `hypit download`): filed as `private-api`, `login: none`. Reason: the default path solves YouTube JS challenges with the bundled EJS solver in Node, and TikTok needs curl_cffi browser TLS impersonation (hypit-ai/hypit #381, #384). Under the task's literal definitions that is signature/fingerprint work outside the official client. Account risk is zero; only IP-level limits. If yt-dlp is classed `public-web` elsewhere, Hypit should follow.
- No-login automated WebView (Kazumi: play page loaded in WebView, media request captured; captcha cookies reused per rule, in memory only): filed as `public-web`, `login: none`, because `browser-session` requires a login state.
- MoneyPrinterTurbo PR #1214 (curl_cffi for Pixabay) was closed unmerged; current code only detects the Cloudflare challenge and logs an error.
- xhs-account-teardown: `public-web`, `login: none`; note detail needs a note-bound `xsec_token` URL copied from the user's logged-in browser, no cookie is sent.
