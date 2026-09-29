# Coffee Order Processor

A standalone Tauri desktop app that converts Shopify coffee-order CSV exports into formatted Excel workbooks for shipping and raw-coffee planning.

## What it does

- Groups line items by `Shipping Name`, including Shopify continuation rows where the name is blank.
- Totals White, Medium, Dark, Espresso, Decaf, and unmapped products for each customer.
- Keeps Dark and Espresso bag counts separate while combining them for the raw Dark estimate.
- Calculates raw pounds as `bag count ÷ roast ratio` using user-adjustable ratios.
- Shows and exports the total number of bags across all coffee types.
- Rounds each roast's raw amount independently.
- Exports a formatted `.xlsx` worksheet with the summary in the header or footer.
- Keeps all customer data local.

## Editable labels and product mappings

The bundled defaults are in [`config/labels.json`](config/labels.json). On first launch, the app creates an editable copy:

- Windows: `%APPDATA%\com.lakecitycoffee.orderprocessor\labels.json`
- macOS: `~/Library/Application Support/com.lakecitycoffee.orderprocessor/labels.json`

Edit `labels` to change UI/Excel wording or `productMappings` to associate an exact Shopify `Lineitem name` with `white`, `medium`, `dark`, `espresso`, or `decaf`. Restart the app after editing. Invalid files produce a warning and the app safely falls back to its bundled defaults.

## Development

Prerequisites:

- Node.js 20 or newer
- Current stable Rust
- Windows: Microsoft C++ Build Tools and WebView2
- macOS: Xcode Command Line Tools

```sh
npm install
npm run tauri dev
```

Run checks:

```sh
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

## Standalone builds

Build each operating system on that operating system:

```sh
npm run tauri build
```

Windows installers are written below `src-tauri/target/release/bundle/`.

For a universal unsigned macOS build supporting both Apple Silicon and Intel:

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm run tauri build -- --target universal-apple-darwin
```

Unsigned macOS apps may be blocked on first launch. For private distribution, the user can control-click the app, choose **Open**, and confirm. Public distribution should use an Apple Developer certificate and notarization.
