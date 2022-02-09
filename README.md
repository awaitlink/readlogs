# Readlogs
Readlogs is an [unofficial](#disclaimer) web app for viewing [Signal](https://signal.org) debug logs without manually downloading or unarchiving them.

To use it, create a debug log via the Signal app on your device as described in [this support article](https://support.signal.org/hc/en-us/articles/360007318591), then open [`readlogs.pages.dev`](https://readlogs.pages.dev/) and paste the URL there.

## Current functionality
- View information and logs from various sections of debug logs, formatted as tables.
- Search logs by setting a minimum log level (e.g. "Warn" to show warnings, errors, and more important log entries, if any) as well as using a (case-insensitive) search query.
- View and download raw debug log files in plaintext (i.e. unarchived).

### Notable behavior
- Log entries that span multiple lines (without introducing a new timestamp and other metadata) are assumed to be one log entry.
- In case of Signal Android, sometimes multiple consecutive log lines repeat the exact same timestamp and metadata. These are collapsed into one entry.
- Some Signal iOS log entries don't seem to have a log level; it's assumed to be `LogLevel::Info`.

## Overview
This repository primarily contains two pieces of software:
- A Rust web app (via [WebAssembly](https://webassembly.org)) in the [`src`](/src) folder (as well as `index.html` and other files in the root).
- A small JavaScript [Cloudflare Worker](https://workers.cloudflare.com) in the [`worker`](/worker) folder.

## How it works
### 1. Parsing the debug log URL
All debug log URLs have one of the below formats. The provided URL is parsed before fetching in order catch any potential copy/paste mistakes earlier, thus minimizing requests to the worker.

```text
https://debuglogs.org/{key}{ext?}
https://debuglogs.org/{platform}/{version}/{key}{ext?}
```

- `platform` is `ios` for Signal iOS, `desktop` for Signal Desktop, and `android` for Signal Android.
- `version` is the Signal app's version.
- `key` is a 64-character string consisting of `a-f` and `0-9`.
- `ext` is `.zip` for Signal iOS, `.gz` for Signal Desktop, and none for Signal Android.

### 2. Fetching
In general, the file is fetched using the worker: it's not possible to fetch directly from `debuglogs.org` due to its [Cross-Origin Resource Sharing](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) policy.

There are some differences in the fetching process between Signal Android/Desktop and Signal iOS:

- **Signal Android/Desktop**

    The file is `gzip`-ped plain text. The worker alters the response to indicate this; presumably, Cloudflare then picks the best compression to deliver the response to your browser (e.g. `brotli`). The browser uncompresses the response and provides it to the app in plain text.

    So all compression work is done by Cloudflare and the browser; neither the worker itself nor the app.

- **Signal iOS**

    The file is a `zip` archive containing multiple files. The worker delivers it unmodified, and the web app itself handles the decompression.

### 3. Parsing and displaying
Each file (there is one for Signal Android/Desktop, but multiple in case of Signal iOS) is parsed by the web app immediately after fetching.

**Note:** Signal Desktop *can* output each log entry in a structured JSON format (if you start it from a terminal and look at the output), however the file submitted to `debuglogs.org` has the log in plaintext format, which is what this projects parses.

## Privacy considerations
Note that debug logs uploaded by the Signal apps already have sensitive information redacted.

### Inferring whether anyone viewed a given debug log

It could be possible to infer that someone has recently viewed a given debug log using this project because of different response times due to potential additional cache (compared to just downloading from `debuglogs.org`) being hit or missed, etc.

This is counteracted by *not* caching the worker's response in Cloudflare's CDN. So, each request to the worker, if deeemed valid in the first place, will always request the debug log directly from `debuglogs.org`.

The response is, however, cached locally in the browser (currently, for 7 days) to avoid repeated requests in case the user is viewing the same debug log multiple times.

## Building the app
1. Install [Yarn](https://yarnpkg.com).
1. Install CSS dependencies:
    ```shell
    yarn install
    ```
1. Install [Rust](https://www.rust-lang.org), for example via [`rustup`](https://rustup.rs).
1. Install [Trunk](https://trunkrs.dev), for example via:
    ```shell
    cargo install trunk
    ```
1. Build the app:
    ```shell
    trunk build
    ```

    Alternatively, serve it locally:
    ```shell
    trunk serve
    ```

    **Note:** Performance of debug builds may be significantly worse than that of release builds. For this reason, when using the app, it's recommended to build/serve in release mode instead:
    ```shell
    trunk build --release
    ```
    ```shell
    trunk serve --release
    ```
1. The built app is now available in the `dist` folder.

## Deploying the worker
1. Follow steps 1–3 of the Cloudflare Workers [Get started guide](https://developers.cloudflare.com/workers/get-started/guide).
1. Switch to the `worker` folder.
1. Edit `wrangler.toml` if necessary (e.g. to change the `name` of the worker).
1. Run the following command to publish your worker:
    ```shell
    wrangler publish
    ```
1. The worker should now be published at `<name>.<your-subdomain>.workers.dev`.

## Disclaimer
This is an unofficial project. It is *not* affilated with the Signal Technology Foundation or Signal Messenger, LLC.
